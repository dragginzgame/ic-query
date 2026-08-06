//! Module: nns::registry::replay::session
//!
//! Responsibility: coordinate exact-target replay and compact provenance commitments.
//! Does not own: source calls, persistence, catalog projection, or assurance promotion.
//! Boundary: ordinary replay pins one target; archive segments advance only after atomic completion.

use super::{
    NnsRegistryReplayError, NnsRegistryReplayLimits, NnsRegistryReplayProgress,
    NnsRegistryReplayState, apply_validated_batch_through, validated_batch_prefix_counts,
};
use crate::nns::registry::{
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    validate_nns_certified_registry_delta_batch,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

const EVIDENCE_CHAIN_DOMAIN: &[u8] = b"ic-query:nns-certified-registry-evidence-chain:v1\0";
const COMPLETE_STATE_DOMAIN: &[u8] = b"ic-query:nns-certified-registry-state:v1\0";

/// Version of the replay evidence-chain and complete-state digest contracts.
pub const NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION: u32 = 1;

///
/// NnsRegistryReplaySessionLimits
///
/// Explicit cumulative evidence and state ceilings for one exact-target replay session.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsRegistryReplaySessionLimits {
    /// Maximum Registry version that may be selected as any exact replay target.
    pub max_registry_versions: u64,
    /// Maximum certified delta batches admitted by the session.
    pub max_batches: u64,
    /// Maximum reported Registry query calls across admitted batches.
    pub max_query_calls: u64,
    /// Maximum reported encoded response bytes across admitted batches.
    pub max_response_bytes: u64,
    /// Payload ceilings enforced on the reconstructed current state.
    pub state: NnsRegistryReplayLimits,
}

impl NnsRegistryReplaySessionLimits {
    /// Create explicit session ceilings without choosing hidden bootstrap defaults.
    #[must_use]
    pub const fn new(
        max_registry_versions: u64,
        max_batches: u64,
        max_query_calls: u64,
        max_response_bytes: u64,
        state: NnsRegistryReplayLimits,
    ) -> Self {
        Self {
            max_registry_versions,
            max_batches,
            max_query_calls,
            max_response_bytes,
            state,
        }
    }
}

///
/// NnsRegistryReplaySession
///
/// Pure cumulative replay coordination pinned to the first batch's certified latest version.
/// Ordinary callers cannot reopen a completed session. Authenticated archive construction may
/// internally advance the target only between explicitly retained complete segments.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsRegistryReplaySession {
    limits: NnsRegistryReplaySessionLimits,
    state: NnsRegistryReplayState,
    selected_version: Option<u64>,
    highest_certified_latest_version: Option<u64>,
    root_key_digest: Option<String>,
    source_endpoints: BTreeSet<String>,
    minimum_certificate_time_nanos: Option<u64>,
    maximum_certificate_time_nanos: Option<u64>,
    evidence_chain_digest: Option<[u8; 32]>,
    complete_state_digest: Option<[u8; 32]>,
    batch_count: u64,
    query_call_count: u64,
    response_bytes: u64,
    applied_mutation_count: u64,
}

impl NnsRegistryReplaySession {
    /// Create a version-zero replay session with explicit cumulative limits.
    #[must_use]
    pub const fn new(limits: NnsRegistryReplaySessionLimits) -> Self {
        Self {
            limits,
            state: NnsRegistryReplayState::new(),
            selected_version: None,
            highest_certified_latest_version: None,
            root_key_digest: None,
            source_endpoints: BTreeSet::new(),
            minimum_certificate_time_nanos: None,
            maximum_certificate_time_nanos: None,
            evidence_chain_digest: None,
            complete_state_digest: None,
            batch_count: 0,
            query_call_count: 0,
            response_bytes: 0,
            applied_mutation_count: 0,
        }
    }

    /// Validate and atomically admit one caller-supplied certified delta batch.
    ///
    /// The first accepted batch selects its certified latest Registry version.
    /// Later reports may observe a newer latest version, but mutations after the
    /// selected version are not applied.
    pub fn apply_batch(
        &mut self,
        request: &NnsCertifiedRegistryDeltaBatchRequest,
        report: &NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsRegistryReplayProgress, NnsRegistryReplayError> {
        validate_nns_certified_registry_delta_batch(request, report)?;
        self.apply_prevalidated_batch(report)
    }

    pub(super) fn apply_prevalidated_batch(
        &mut self,
        report: &NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsRegistryReplayProgress, NnsRegistryReplayError> {
        self.apply_prevalidated_batch_with_target(report, false)
    }

    pub(super) fn apply_prevalidated_extension_batch(
        &mut self,
        report: &NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<NnsRegistryReplayProgress, NnsRegistryReplayError> {
        self.apply_prevalidated_batch_with_target(report, true)
    }

    fn apply_prevalidated_batch_with_target(
        &mut self,
        report: &NnsCertifiedRegistryDeltaBatchReport,
        select_new_target: bool,
    ) -> Result<NnsRegistryReplayProgress, NnsRegistryReplayError> {
        if self.state.through_version() != report.requested_version {
            return Err(NnsRegistryReplayError::VersionMismatch {
                state_version: self.state.through_version(),
                requested_version: report.requested_version,
            });
        }
        if select_new_target && !self.is_complete() {
            return Err(
                NnsRegistryReplayError::ArchiveExtensionRequiresCompleteSegment {
                    selected_version: self.selected_version,
                    through_version: self.state.through_version(),
                },
            );
        }
        if !select_new_target && let Some(selected_version) = self.selected_version {
            if self.state.through_version() == selected_version {
                return Err(NnsRegistryReplayError::SessionComplete { selected_version });
            }
            if report.certified_latest_version < selected_version {
                return Err(NnsRegistryReplayError::CertifiedVersionRegressed {
                    selected_version,
                    certified_latest_version: report.certified_latest_version,
                });
            }
        }
        if let Some(expected_root_key_digest) = &self.root_key_digest
            && expected_root_key_digest != &report.certification.root_key_digest
        {
            return Err(NnsRegistryReplayError::RootKeyDigestMismatch {
                expected_root_key_digest: expected_root_key_digest.clone(),
                actual_root_key_digest: report.certification.root_key_digest.clone(),
            });
        }

        let selected_version = if select_new_target {
            report.certified_latest_version
        } else {
            self.selected_version
                .unwrap_or(report.certified_latest_version)
        };
        enforce_session_limit(
            "selected Registry versions",
            selected_version,
            self.limits.max_registry_versions,
        )?;
        let candidate_batch_count = checked_add(self.batch_count, 1)?;
        enforce_session_limit(
            "batch count",
            candidate_batch_count,
            self.limits.max_batches,
        )?;
        let candidate_query_call_count =
            checked_add(self.query_call_count, report.query_call_count)?;
        enforce_session_limit(
            "query call count",
            candidate_query_call_count,
            self.limits.max_query_calls,
        )?;
        let report_response_bytes =
            u64::try_from(report.response_bytes).map_err(|_| NnsRegistryReplayError::Accounting)?;
        let candidate_response_bytes = checked_add(self.response_bytes, report_response_bytes)?;
        enforce_session_limit(
            "response bytes",
            candidate_response_bytes,
            self.limits.max_response_bytes,
        )?;
        let (_, batch_applied_mutation_count) =
            validated_batch_prefix_counts(report, selected_version)?;
        let batch_applied_mutation_count = u64::try_from(batch_applied_mutation_count)
            .map_err(|_| NnsRegistryReplayError::Accounting)?;
        let candidate_applied_mutation_count =
            checked_add(self.applied_mutation_count, batch_applied_mutation_count)?;
        let candidate_provenance = self.candidate_provenance(report)?;

        let progress = apply_validated_batch_through(
            &mut self.state,
            report,
            selected_version,
            self.limits.state,
        )?;
        self.applied_mutation_count = candidate_applied_mutation_count;
        self.selected_version = Some(selected_version);
        self.highest_certified_latest_version = Some(
            self.highest_certified_latest_version
                .map_or(report.certified_latest_version, |version| {
                    version.max(report.certified_latest_version)
                }),
        );
        self.root_key_digest = Some(report.certification.root_key_digest.clone());
        self.publish_provenance(candidate_provenance, selected_version);
        self.batch_count = candidate_batch_count;
        self.query_call_count = candidate_query_call_count;
        self.response_bytes = candidate_response_bytes;
        Ok(progress)
    }

    fn candidate_provenance(
        &self,
        report: &NnsCertifiedRegistryDeltaBatchReport,
    ) -> Result<SessionProvenanceCandidate, NnsRegistryReplayError> {
        let evidence_chain_digest = chained_evidence_digest(self.evidence_chain_digest, report)?;
        let mut source_endpoints = self.source_endpoints.clone();
        source_endpoints.insert(report.source_endpoint.clone());
        let certificate_time_nanos = report.certification.certificate_time_nanos;
        Ok(SessionProvenanceCandidate {
            source_endpoints,
            minimum_certificate_time_nanos: self
                .minimum_certificate_time_nanos
                .map_or(certificate_time_nanos, |time| {
                    time.min(certificate_time_nanos)
                }),
            maximum_certificate_time_nanos: self
                .maximum_certificate_time_nanos
                .map_or(certificate_time_nanos, |time| {
                    time.max(certificate_time_nanos)
                }),
            evidence_chain_digest,
        })
    }

    fn publish_provenance(&mut self, candidate: SessionProvenanceCandidate, selected_version: u64) {
        self.source_endpoints = candidate.source_endpoints;
        self.minimum_certificate_time_nanos = Some(candidate.minimum_certificate_time_nanos);
        self.maximum_certificate_time_nanos = Some(candidate.maximum_certificate_time_nanos);
        self.evidence_chain_digest = Some(candidate.evidence_chain_digest);
        self.complete_state_digest = (self.state.through_version() == selected_version)
            .then(|| replay_state_digest(&self.state));
    }

    pub(super) fn ensure_next_source_call_capacity(
        &self,
        maximum_batch_query_calls: u64,
        maximum_batch_response_bytes: u64,
    ) -> Result<(), NnsRegistryReplayError> {
        self.ensure_source_call_capacity(
            maximum_batch_query_calls,
            maximum_batch_response_bytes,
            false,
        )
    }

    pub(super) fn ensure_next_extension_source_call_capacity(
        &self,
        maximum_batch_query_calls: u64,
        maximum_batch_response_bytes: u64,
    ) -> Result<(), NnsRegistryReplayError> {
        self.ensure_source_call_capacity(
            maximum_batch_query_calls,
            maximum_batch_response_bytes,
            true,
        )
    }

    fn ensure_source_call_capacity(
        &self,
        maximum_batch_query_calls: u64,
        maximum_batch_response_bytes: u64,
        permit_completed_segment: bool,
    ) -> Result<(), NnsRegistryReplayError> {
        if !permit_completed_segment
            && let Some(selected_version) = self.selected_version
            && self.state.through_version() == selected_version
        {
            return Err(NnsRegistryReplayError::SessionComplete { selected_version });
        }
        enforce_session_limit(
            "batch count",
            checked_add(self.batch_count, 1)?,
            self.limits.max_batches,
        )?;
        enforce_session_limit(
            "query call count",
            checked_add(self.query_call_count, maximum_batch_query_calls)?,
            self.limits.max_query_calls,
        )?;
        enforce_session_limit(
            "response bytes",
            checked_add(self.response_bytes, maximum_batch_response_bytes)?,
            self.limits.max_response_bytes,
        )
    }

    /// Return the explicit limits governing this session.
    #[must_use]
    pub const fn limits(&self) -> NnsRegistryReplaySessionLimits {
        self.limits
    }

    /// Return the reconstructed Registry state.
    #[must_use]
    pub const fn state(&self) -> &NnsRegistryReplayState {
        &self.state
    }

    /// Consume the session and return its reconstructed Registry state.
    #[must_use]
    pub fn into_state(self) -> NnsRegistryReplayState {
        self.state
    }

    /// Return the current exact target selected by this replay or its latest archive segment.
    #[must_use]
    pub const fn selected_version(&self) -> Option<u64> {
        self.selected_version
    }

    /// Return the highest certified latest version observed without moving the target.
    #[must_use]
    pub const fn highest_certified_latest_version(&self) -> Option<u64> {
        self.highest_certified_latest_version
    }

    /// Return whether the selected exact target has been fully reconstructed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.selected_version == Some(self.state.through_version())
    }

    /// Return the common root-key digest retained from admitted batches.
    #[must_use]
    pub fn root_key_digest(&self) -> Option<&str> {
        self.root_key_digest.as_deref()
    }

    /// Iterate distinct validated source endpoint strings in canonical order.
    pub fn source_endpoints(&self) -> impl ExactSizeIterator<Item = &str> {
        self.source_endpoints.iter().map(String::as_str)
    }

    /// Return the earliest certificate time retained from admitted reports.
    #[must_use]
    pub const fn minimum_certificate_time_nanos(&self) -> Option<u64> {
        self.minimum_certificate_time_nanos
    }

    /// Return the latest certificate time retained from admitted reports.
    #[must_use]
    pub const fn maximum_certificate_time_nanos(&self) -> Option<u64> {
        self.maximum_certificate_time_nanos
    }

    /// Return the domain-separated digest chain over every admitted validated report.
    ///
    /// This commits to report contents and ordering. For custom sources it does
    /// not replace the source's responsibility to authenticate raw evidence.
    #[must_use]
    pub const fn evidence_chain_digest(&self) -> Option<[u8; 32]> {
        self.evidence_chain_digest
    }

    /// Return the canonical reconstructed-state digest only after exact completion.
    #[must_use]
    pub const fn complete_state_digest(&self) -> Option<[u8; 32]> {
        self.complete_state_digest
    }

    /// Return the number of admitted certified delta batches.
    #[must_use]
    pub const fn batch_count(&self) -> u64 {
        self.batch_count
    }

    /// Return reported Registry query calls across admitted batches.
    #[must_use]
    pub const fn query_call_count(&self) -> u64 {
        self.query_call_count
    }

    /// Return reported encoded response bytes across admitted batches.
    #[must_use]
    pub const fn response_bytes(&self) -> u64 {
        self.response_bytes
    }

    /// Return committed mutations applied through the selected target.
    #[must_use]
    pub const fn applied_mutation_count(&self) -> u64 {
        self.applied_mutation_count
    }
}

struct SessionProvenanceCandidate {
    source_endpoints: BTreeSet<String>,
    minimum_certificate_time_nanos: u64,
    maximum_certificate_time_nanos: u64,
    evidence_chain_digest: [u8; 32],
}

fn chained_evidence_digest(
    previous: Option<[u8; 32]>,
    report: &NnsCertifiedRegistryDeltaBatchReport,
) -> Result<[u8; 32], NnsRegistryReplayError> {
    let mut hasher = Sha256::new();
    hasher.update(EVIDENCE_CHAIN_DOMAIN);
    match previous {
        Some(digest) => {
            hasher.update([1]);
            hasher.update(digest);
        }
        None => hasher.update([0]),
    }
    serde_json::to_writer(Sha256Writer(&mut hasher), report).map_err(|error| {
        NnsRegistryReplayError::EvidenceEncoding {
            reason: error.to_string(),
        }
    })?;
    Ok(hasher.finalize().into())
}

fn replay_state_digest(state: &NnsRegistryReplayState) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(COMPLETE_STATE_DOMAIN);
    hasher.update(state.through_version().to_be_bytes());
    hasher.update(encoded_usize(state.entry_count()));
    hasher.update(encoded_usize(state.content_bytes()));
    for (key, value) in state.entries() {
        hash_bytes(&mut hasher, key);
        hash_bytes(&mut hasher, value.value());
        hasher.update(value.last_mutation_version().to_be_bytes());
        hasher.update(value.timestamp_nanoseconds().to_be_bytes());
    }
    hasher.finalize().into()
}

fn hash_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(encoded_usize(bytes.len()));
    hasher.update(bytes);
}

fn encoded_usize(value: usize) -> [u8; 16] {
    let native = value.to_be_bytes();
    let mut encoded = [0; 16];
    encoded[16 - native.len()..].copy_from_slice(&native);
    encoded
}

struct Sha256Writer<'a>(&'a mut Sha256);

impl Write for Sha256Writer<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.update(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn checked_add(left: u64, right: u64) -> Result<u64, NnsRegistryReplayError> {
    left.checked_add(right)
        .ok_or(NnsRegistryReplayError::Accounting)
}

const fn enforce_session_limit(
    field: &'static str,
    actual: u64,
    maximum: u64,
) -> Result<(), NnsRegistryReplayError> {
    if actual > maximum {
        Err(NnsRegistryReplayError::SessionLimitExceeded {
            field,
            maximum,
            actual,
        })
    } else {
        Ok(())
    }
}
