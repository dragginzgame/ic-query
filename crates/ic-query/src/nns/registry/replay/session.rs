//! Module: nns::registry::replay::session
//!
//! Responsibility: coordinate cumulative, exact-target application of caller-supplied batches.
//! Does not own: source calls, persistence, catalog projection, or assurance promotion.
//! Boundary: the first valid batch pins the target; later batches cannot move it.

use super::{
    NnsRegistryReplayError, NnsRegistryReplayLimits, NnsRegistryReplayProgress,
    NnsRegistryReplayState, apply_validated_batch_through, validated_batch_prefix_counts,
};
use crate::nns::registry::{
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    validate_nns_certified_registry_delta_batch,
};

///
/// NnsRegistryReplaySessionLimits
///
/// Explicit cumulative evidence and state ceilings for one exact-target replay session.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsRegistryReplaySessionLimits {
    /// Maximum Registry version that may be selected from the first batch.
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
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsRegistryReplaySession {
    limits: NnsRegistryReplaySessionLimits,
    state: NnsRegistryReplayState,
    selected_version: Option<u64>,
    highest_certified_latest_version: Option<u64>,
    root_key_digest: Option<String>,
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
        if self.state.through_version() != report.requested_version {
            return Err(NnsRegistryReplayError::VersionMismatch {
                state_version: self.state.through_version(),
                requested_version: report.requested_version,
            });
        }
        if let Some(selected_version) = self.selected_version {
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

        let selected_version = self
            .selected_version
            .unwrap_or(report.certified_latest_version);
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
        self.batch_count = candidate_batch_count;
        self.query_call_count = candidate_query_call_count;
        self.response_bytes = candidate_response_bytes;
        Ok(progress)
    }

    pub(super) fn ensure_next_source_call_capacity(
        &self,
        maximum_batch_query_calls: u64,
        maximum_batch_response_bytes: u64,
    ) -> Result<(), NnsRegistryReplayError> {
        if let Some(selected_version) = self.selected_version
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

    /// Return the exact target selected by the first accepted batch.
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
