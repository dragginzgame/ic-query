//! Module: nns::registry::replay::archive
//!
//! Responsibility: describe and validate bounded retained certified Registry evidence.
//! Does not own: filesystem paths, publication, cache policy, source calls, or catalog assurance.
//! Boundary: manifests are indexes, not authority; reports must be reauthenticated on every load.

use super::{
    NnsAuthenticatedRegistryReplayBuilder, NnsAuthenticatedRegistryReplaySession,
    NnsRegistryReplayError, NnsRegistryReplayProgress, NnsRegistryReplaySessionLimits,
    validated_batch_prefix_counts,
};
use crate::{
    hex::{hex_bytes, is_lowercase_hex},
    http_endpoint::parse_http_endpoint,
    nns::registry::{
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION, NnsAuthenticatedRegistryDeltaBatch,
        NnsCertifiedRegistryDeltaBatchReport,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use thiserror::Error as ThisError;

/// Version of the retained certified Registry archive-manifest contract.
pub const NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION: u32 = 1;

///
/// NnsCertifiedRegistryArchiveLimits
///
/// Caller-selected ceilings for canonical retained-report encodings.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryArchiveLimits {
    /// Maximum retained batch descriptors in one archive manifest.
    pub max_batches: u64,
    /// Maximum canonical JSON bytes for one retained batch report.
    pub max_batch_report_bytes: u64,
    /// Maximum canonical JSON bytes across every retained batch report.
    pub max_total_report_bytes: u64,
}

impl NnsCertifiedRegistryArchiveLimits {
    /// Create explicit archive ceilings without selecting hidden defaults.
    #[must_use]
    pub const fn new(
        max_batches: u64,
        max_batch_report_bytes: u64,
        max_total_report_bytes: u64,
    ) -> Self {
        Self {
            max_batches,
            max_batch_report_bytes,
            max_total_report_bytes,
        }
    }
}

///
/// NnsCertifiedRegistryArchiveBatchDescriptor
///
/// Canonical position, accounting, and content commitment for one retained report object.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsCertifiedRegistryArchiveBatchDescriptor {
    /// Zero-based canonical position in the archive.
    pub ordinal: u64,
    /// Registry version after which this report requested changes.
    pub requested_version: u64,
    /// First version carried by the report, when any.
    pub first_version: Option<u64>,
    /// Last version carried by the report, when any.
    pub last_version: Option<u64>,
    /// Replay position after applying this report through the selected target.
    pub applied_through_version: u64,
    /// Latest Registry version authenticated by this report.
    pub certified_latest_version: u64,
    /// Registry query calls reported by this batch.
    pub query_call_count: u64,
    /// Encoded Registry response bytes reported by this batch.
    pub response_bytes: u64,
    /// Mutations from this batch applied through the selected target.
    pub applied_mutation_count: u64,
    /// Canonical compact-JSON byte length of the retained report object.
    pub report_bytes: u64,
    /// SHA-256 of the exact canonical compact-JSON retained report object.
    pub report_sha256: String,
}

///
/// NnsCertifiedRegistryArchiveManifest
///
/// Versioned index derived from a complete authenticated Registry replay sequence.
/// A deserialized manifest is untrusted metadata. Authority is recovered only by
/// loading each exact report object, reauthenticating it, and replaying the complete
/// sequence before comparing the recomputed manifest.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsCertifiedRegistryArchiveManifest {
    /// Archive-manifest schema version.
    pub schema_version: u32,
    /// Certified Registry delta-report schema required for every retained object.
    pub delta_report_schema_version: u32,
    /// Replay provenance schema governing evidence-chain and complete-state digests.
    pub replay_provenance_schema_version: u32,
    /// Network identity; schema 1 supports only mainnet `ic`.
    pub network: String,
    /// Canonical mainnet Registry canister principal.
    pub registry_canister_id: String,
    /// Exact Registry version reconstructed by the complete sequence.
    pub selected_version: u64,
    /// Number of retained batch report objects.
    pub batch_count: u64,
    /// Canonical compact-JSON bytes across retained report objects.
    pub total_report_bytes: u64,
    /// Registry query calls reported across retained batches.
    pub query_call_count: u64,
    /// Encoded Registry response bytes reported across retained batches.
    pub response_bytes: u64,
    /// Mutations applied through the selected exact target.
    pub applied_mutation_count: u64,
    /// Common trusted root-key digest from the authenticated sequence.
    pub root_key_digest: String,
    /// Domain-separated digest chain over the ordered report contents.
    pub evidence_chain_digest: String,
    /// Canonical digest of the reconstructed exact-target Registry state.
    pub complete_state_digest: String,
    /// Earliest authenticated certificate time in nanoseconds.
    pub minimum_certificate_time_nanos: u64,
    /// Latest authenticated certificate time in nanoseconds.
    pub maximum_certificate_time_nanos: u64,
    /// Distinct report source endpoints in canonical lexical order.
    pub source_endpoints: Vec<String>,
    /// Retained report descriptors in strict replay order.
    pub batches: Vec<NnsCertifiedRegistryArchiveBatchDescriptor>,
}

///
/// NnsCertifiedRegistryArchiveManifestBuilder
///
/// Bounded manifest construction that accepts only locally authenticated report capabilities.
///

#[derive(Debug)]
pub struct NnsCertifiedRegistryArchiveManifestBuilder {
    replay: NnsAuthenticatedRegistryReplayBuilder,
    limits: NnsCertifiedRegistryArchiveLimits,
    total_report_bytes: u64,
    batches: Vec<NnsCertifiedRegistryArchiveBatchDescriptor>,
}

impl NnsCertifiedRegistryArchiveManifestBuilder {
    /// Create an empty version-zero archive-manifest builder with explicit limits.
    #[must_use]
    pub const fn new(
        replay_limits: NnsRegistryReplaySessionLimits,
        archive_limits: NnsCertifiedRegistryArchiveLimits,
    ) -> Self {
        Self {
            replay: NnsAuthenticatedRegistryReplayBuilder::new(replay_limits),
            limits: archive_limits,
            total_report_bytes: 0,
            batches: Vec::new(),
        }
    }

    /// Authenticate archive accounting and atomically apply one retained report batch.
    pub fn apply_batch(
        &mut self,
        batch: &NnsAuthenticatedRegistryDeltaBatch<'_>,
    ) -> Result<NnsRegistryReplayProgress, NnsCertifiedRegistryArchiveError> {
        let ordinal = u64::try_from(self.batches.len())
            .map_err(|_| NnsCertifiedRegistryArchiveError::Accounting)?;
        enforce_archive_limit(
            "batch count",
            checked_add(ordinal, 1)?,
            self.limits.max_batches,
        )?;
        let report_encoding = canonical_report_encoding(batch.report())?;
        enforce_archive_limit(
            "batch report bytes",
            report_encoding.bytes,
            self.limits.max_batch_report_bytes,
        )?;
        let candidate_total_report_bytes =
            checked_add(self.total_report_bytes, report_encoding.bytes)?;
        enforce_archive_limit(
            "total report bytes",
            candidate_total_report_bytes,
            self.limits.max_total_report_bytes,
        )?;

        let report = batch.report();
        let response_bytes = u64::try_from(report.response_bytes)
            .map_err(|_| NnsCertifiedRegistryArchiveError::Accounting)?;
        let selected_version = self
            .replay
            .replay_session()
            .selected_version()
            .unwrap_or(report.certified_latest_version);
        let (_, applied_mutation_count) = validated_batch_prefix_counts(report, selected_version)?;
        let applied_mutation_count = u64::try_from(applied_mutation_count)
            .map_err(|_| NnsCertifiedRegistryArchiveError::Accounting)?;
        let progress = self.replay.apply_batch(batch)?;
        self.batches
            .push(NnsCertifiedRegistryArchiveBatchDescriptor {
                ordinal,
                requested_version: report.requested_version,
                first_version: report.first_version,
                last_version: report.last_version,
                applied_through_version: progress.through_version,
                certified_latest_version: report.certified_latest_version,
                query_call_count: report.query_call_count,
                response_bytes,
                applied_mutation_count,
                report_bytes: report_encoding.bytes,
                report_sha256: hex_bytes(&report_encoding.sha256),
            });
        self.total_report_bytes = candidate_total_report_bytes;
        Ok(progress)
    }

    /// Return replay progress without exposing an ordinary mutable replay session.
    #[must_use]
    pub const fn replay_session(&self) -> &super::NnsRegistryReplaySession {
        self.replay.replay_session()
    }

    /// Finish one complete manifest and retain the authenticated replay capability.
    pub fn finish(
        self,
    ) -> Result<
        (
            NnsCertifiedRegistryArchiveManifest,
            NnsAuthenticatedRegistryReplaySession,
        ),
        NnsCertifiedRegistryArchiveError,
    > {
        let authenticated = self.replay.into_authenticated_replay_session()?;
        let session = authenticated.replay_session();
        let manifest = NnsCertifiedRegistryArchiveManifest {
            schema_version: NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION,
            delta_report_schema_version: NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION,
            replay_provenance_schema_version: super::NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION,
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            selected_version: required_session_value(
                "selected_version",
                session.selected_version(),
            )?,
            batch_count: session.batch_count(),
            total_report_bytes: self.total_report_bytes,
            query_call_count: session.query_call_count(),
            response_bytes: session.response_bytes(),
            applied_mutation_count: session.applied_mutation_count(),
            root_key_digest: required_session_text("root_key_digest", session.root_key_digest())?,
            evidence_chain_digest: hex_bytes(&required_session_value(
                "evidence_chain_digest",
                session.evidence_chain_digest(),
            )?),
            complete_state_digest: hex_bytes(&required_session_value(
                "complete_state_digest",
                session.complete_state_digest(),
            )?),
            minimum_certificate_time_nanos: required_session_value(
                "minimum_certificate_time_nanos",
                session.minimum_certificate_time_nanos(),
            )?,
            maximum_certificate_time_nanos: required_session_value(
                "maximum_certificate_time_nanos",
                session.maximum_certificate_time_nanos(),
            )?,
            source_endpoints: session.source_endpoints().map(str::to_string).collect(),
            batches: self.batches,
        };
        validate_nns_certified_registry_archive_manifest(&manifest, self.limits)?;
        Ok((manifest, authenticated))
    }
}

///
/// NnsCertifiedRegistryArchiveError
///
/// Typed failures from bounded manifest construction and structural validation.
///

#[derive(Debug, ThisError)]
pub enum NnsCertifiedRegistryArchiveError {
    /// Authenticated report replay failed before manifest state was published.
    #[error(transparent)]
    Replay(#[from] NnsRegistryReplayError),

    /// Canonical report JSON could not be encoded.
    #[error("certified Registry archive report encoding failed: {reason}")]
    ReportEncoding {
        /// Serialization failure description.
        reason: String,
    },

    /// Archive report-object accounting exceeded an explicit caller ceiling.
    #[error("certified Registry archive {field} would be {actual}; caller maximum is {maximum}")]
    LimitExceeded {
        /// Bounded archive resource.
        field: &'static str,
        /// Caller-selected ceiling.
        maximum: u64,
        /// Candidate or declared amount.
        actual: u64,
    },

    /// A manifest is malformed or internally inconsistent.
    #[error("invalid certified Registry archive manifest: {reason}")]
    InvalidManifest {
        /// Deterministic validation failure.
        reason: String,
    },

    /// Integer accounting could not be represented safely.
    #[error("certified Registry archive accounting overflow")]
    Accounting,
}

/// Validate an archive manifest structurally without trusting it as Registry authority.
pub fn validate_nns_certified_registry_archive_manifest(
    manifest: &NnsCertifiedRegistryArchiveManifest,
    limits: NnsCertifiedRegistryArchiveLimits,
) -> Result<(), NnsCertifiedRegistryArchiveError> {
    validate_manifest_identity(manifest)?;
    let batch_count = u64::try_from(manifest.batches.len())
        .map_err(|_| NnsCertifiedRegistryArchiveError::Accounting)?;
    if manifest.batch_count != batch_count {
        return Err(invalid_manifest(format!(
            "batch_count must equal batches length; expected {batch_count}, got {}",
            manifest.batch_count
        )));
    }
    if batch_count == 0 {
        return Err(invalid_manifest("batches must not be empty"));
    }
    enforce_archive_limit("batch count", batch_count, limits.max_batches)?;

    let mut totals = ManifestTotals::default();
    let mut expected_requested_version = 0_u64;
    for (index, batch) in manifest.batches.iter().enumerate() {
        if index > 0 && expected_requested_version == manifest.selected_version {
            return Err(invalid_manifest(
                "batches must stop after reaching selected_version",
            ));
        }
        validate_batch_descriptor(
            batch,
            index,
            expected_requested_version,
            manifest.selected_version,
            limits,
        )?;
        expected_requested_version = batch.applied_through_version;
        totals.add(batch)?;
    }
    if expected_requested_version != manifest.selected_version {
        return Err(invalid_manifest(format!(
            "final applied version must equal selected_version {}; got {expected_requested_version}",
            manifest.selected_version
        )));
    }
    totals.validate(manifest)?;
    enforce_archive_limit(
        "total report bytes",
        manifest.total_report_bytes,
        limits.max_total_report_bytes,
    )?;
    validate_manifest_provenance(manifest)
}

fn validate_manifest_identity(
    manifest: &NnsCertifiedRegistryArchiveManifest,
) -> Result<(), NnsCertifiedRegistryArchiveError> {
    if manifest.schema_version != NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION {
        return Err(invalid_manifest(format!(
            "schema_version must be {}; got {}",
            NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION, manifest.schema_version
        )));
    }
    if manifest.delta_report_schema_version != NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION {
        return Err(invalid_manifest(format!(
            "delta_report_schema_version must be {}; got {}",
            NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION, manifest.delta_report_schema_version
        )));
    }
    if manifest.replay_provenance_schema_version
        != super::NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION
    {
        return Err(invalid_manifest(format!(
            "replay_provenance_schema_version must be {}; got {}",
            super::NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION,
            manifest.replay_provenance_schema_version
        )));
    }
    if manifest.network != MAINNET_NETWORK {
        return Err(invalid_manifest(format!(
            "network must be {MAINNET_NETWORK:?}; got {:?}",
            manifest.network
        )));
    }
    if manifest.registry_canister_id != MAINNET_REGISTRY_CANISTER_ID {
        return Err(invalid_manifest(format!(
            "registry_canister_id must be {MAINNET_REGISTRY_CANISTER_ID:?}; got {:?}",
            manifest.registry_canister_id
        )));
    }
    Ok(())
}

fn validate_batch_descriptor(
    batch: &NnsCertifiedRegistryArchiveBatchDescriptor,
    index: usize,
    expected_requested_version: u64,
    selected_version: u64,
    limits: NnsCertifiedRegistryArchiveLimits,
) -> Result<(), NnsCertifiedRegistryArchiveError> {
    let ordinal = u64::try_from(index).map_err(|_| NnsCertifiedRegistryArchiveError::Accounting)?;
    if batch.ordinal != ordinal {
        return Err(invalid_manifest(format!(
            "batches[{index}].ordinal must be {ordinal}; got {}",
            batch.ordinal
        )));
    }
    if batch.requested_version != expected_requested_version {
        return Err(invalid_manifest(format!(
            "batches[{index}].requested_version must be {expected_requested_version}; got {}",
            batch.requested_version
        )));
    }
    if index == 0 && batch.certified_latest_version != selected_version {
        return Err(invalid_manifest(format!(
            "batches[0].certified_latest_version must select target {selected_version}; got {}",
            batch.certified_latest_version
        )));
    }
    if batch.certified_latest_version < selected_version {
        return Err(invalid_manifest(format!(
            "batches[{index}].certified_latest_version precedes selected_version"
        )));
    }
    validate_descriptor_versions(batch, index, selected_version)?;
    if batch.query_call_count == 0 {
        return Err(invalid_manifest(format!(
            "batches[{index}].query_call_count must be positive"
        )));
    }
    if batch.response_bytes == 0 {
        return Err(invalid_manifest(format!(
            "batches[{index}].response_bytes must be positive"
        )));
    }
    if batch.report_bytes == 0 {
        return Err(invalid_manifest(format!(
            "batches[{index}].report_bytes must be positive"
        )));
    }
    enforce_archive_limit(
        "batch report bytes",
        batch.report_bytes,
        limits.max_batch_report_bytes,
    )?;
    validate_digest(
        &format!("batches[{index}].report_sha256"),
        &batch.report_sha256,
    )
}

fn validate_descriptor_versions(
    batch: &NnsCertifiedRegistryArchiveBatchDescriptor,
    index: usize,
    selected_version: u64,
) -> Result<(), NnsCertifiedRegistryArchiveError> {
    match (batch.first_version, batch.last_version) {
        (None, None) => {
            if batch.requested_version != batch.certified_latest_version {
                return Err(invalid_manifest(format!(
                    "batches[{index}] omits versions before its certified latest version"
                )));
            }
        }
        (Some(first), Some(last)) => {
            let expected_first = checked_add(batch.requested_version, 1)?;
            if first != expected_first || last < first {
                return Err(invalid_manifest(format!(
                    "batches[{index}] version bounds are not a contiguous continuation"
                )));
            }
            if last > batch.certified_latest_version {
                return Err(invalid_manifest(format!(
                    "batches[{index}].last_version exceeds certified_latest_version"
                )));
            }
        }
        _ => {
            return Err(invalid_manifest(format!(
                "batches[{index}] must provide both first_version and last_version or neither"
            )));
        }
    }
    let visible_through = batch.last_version.unwrap_or(batch.requested_version);
    let expected_applied_through = visible_through.min(selected_version);
    if batch.applied_through_version != expected_applied_through {
        return Err(invalid_manifest(format!(
            "batches[{index}].applied_through_version must be {expected_applied_through}; got {}",
            batch.applied_through_version
        )));
    }
    Ok(())
}

fn validate_manifest_provenance(
    manifest: &NnsCertifiedRegistryArchiveManifest,
) -> Result<(), NnsCertifiedRegistryArchiveError> {
    validate_digest("root_key_digest", &manifest.root_key_digest)?;
    validate_digest("evidence_chain_digest", &manifest.evidence_chain_digest)?;
    validate_digest("complete_state_digest", &manifest.complete_state_digest)?;
    if manifest.minimum_certificate_time_nanos > manifest.maximum_certificate_time_nanos {
        return Err(invalid_manifest(
            "minimum_certificate_time_nanos exceeds maximum_certificate_time_nanos",
        ));
    }
    if manifest.source_endpoints.is_empty() {
        return Err(invalid_manifest("source_endpoints must not be empty"));
    }
    let mut previous = None;
    for (index, endpoint) in manifest.source_endpoints.iter().enumerate() {
        parse_http_endpoint(endpoint).map_err(|reason| {
            invalid_manifest(format!("source_endpoints[{index}] is invalid: {reason}"))
        })?;
        if previous.is_some_and(|previous| previous >= endpoint.as_str()) {
            return Err(invalid_manifest(
                "source_endpoints must be unique and in strict lexical order",
            ));
        }
        previous = Some(endpoint.as_str());
    }
    Ok(())
}

#[derive(Default)]
struct ManifestTotals {
    report_bytes: u64,
    query_call_count: u64,
    response_bytes: u64,
    applied_mutation_count: u64,
}

impl ManifestTotals {
    fn add(
        &mut self,
        batch: &NnsCertifiedRegistryArchiveBatchDescriptor,
    ) -> Result<(), NnsCertifiedRegistryArchiveError> {
        self.report_bytes = checked_add(self.report_bytes, batch.report_bytes)?;
        self.query_call_count = checked_add(self.query_call_count, batch.query_call_count)?;
        self.response_bytes = checked_add(self.response_bytes, batch.response_bytes)?;
        self.applied_mutation_count =
            checked_add(self.applied_mutation_count, batch.applied_mutation_count)?;
        Ok(())
    }

    fn validate(
        self,
        manifest: &NnsCertifiedRegistryArchiveManifest,
    ) -> Result<(), NnsCertifiedRegistryArchiveError> {
        validate_total(
            "total_report_bytes",
            self.report_bytes,
            manifest.total_report_bytes,
        )?;
        validate_total(
            "query_call_count",
            self.query_call_count,
            manifest.query_call_count,
        )?;
        validate_total(
            "response_bytes",
            self.response_bytes,
            manifest.response_bytes,
        )?;
        validate_total(
            "applied_mutation_count",
            self.applied_mutation_count,
            manifest.applied_mutation_count,
        )
    }
}

fn canonical_report_encoding(
    report: &NnsCertifiedRegistryDeltaBatchReport,
) -> Result<CanonicalReportEncoding, NnsCertifiedRegistryArchiveError> {
    let mut writer = HashingWriter::default();
    serde_json::to_writer(&mut writer, report).map_err(|error| {
        NnsCertifiedRegistryArchiveError::ReportEncoding {
            reason: error.to_string(),
        }
    })?;
    Ok(CanonicalReportEncoding {
        bytes: writer.bytes,
        sha256: writer.hasher.finalize().into(),
    })
}

struct CanonicalReportEncoding {
    bytes: u64,
    sha256: [u8; 32],
}

#[derive(Default)]
struct HashingWriter {
    hasher: Sha256,
    bytes: u64,
}

impl Write for HashingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length = u64::try_from(buffer.len())
            .map_err(|_| io::Error::other("buffer length exceeds u64"))?;
        self.bytes = self
            .bytes
            .checked_add(length)
            .ok_or_else(|| io::Error::other("encoded report length exceeds u64"))?;
        self.hasher.update(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn required_session_value<T>(
    field: &'static str,
    value: Option<T>,
) -> Result<T, NnsCertifiedRegistryArchiveError> {
    value.ok_or_else(|| invalid_manifest(format!("complete replay omitted {field}")))
}

fn required_session_text(
    field: &'static str,
    value: Option<&str>,
) -> Result<String, NnsCertifiedRegistryArchiveError> {
    required_session_value(field, value).map(str::to_string)
}

fn validate_digest(field: &str, value: &str) -> Result<(), NnsCertifiedRegistryArchiveError> {
    if value.len() != 64 || !is_lowercase_hex(value) {
        return Err(invalid_manifest(format!(
            "{field} must be exactly 32 bytes of lowercase hexadecimal"
        )));
    }
    Ok(())
}

fn validate_total(
    field: &'static str,
    expected: u64,
    actual: u64,
) -> Result<(), NnsCertifiedRegistryArchiveError> {
    if actual != expected {
        return Err(invalid_manifest(format!(
            "{field} must equal descriptor total {expected}; got {actual}"
        )));
    }
    Ok(())
}

fn checked_add(left: u64, right: u64) -> Result<u64, NnsCertifiedRegistryArchiveError> {
    left.checked_add(right)
        .ok_or(NnsCertifiedRegistryArchiveError::Accounting)
}

const fn enforce_archive_limit(
    field: &'static str,
    actual: u64,
    maximum: u64,
) -> Result<(), NnsCertifiedRegistryArchiveError> {
    if actual > maximum {
        Err(NnsCertifiedRegistryArchiveError::LimitExceeded {
            field,
            maximum,
            actual,
        })
    } else {
        Ok(())
    }
}

fn invalid_manifest(reason: impl Into<String>) -> NnsCertifiedRegistryArchiveError {
    NnsCertifiedRegistryArchiveError::InvalidManifest {
        reason: reason.into(),
    }
}
