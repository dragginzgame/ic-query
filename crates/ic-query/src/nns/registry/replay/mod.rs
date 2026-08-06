//! Module: nns::registry::replay
//!
//! Responsibility: apply certified Registry batches, retain authentication, and project replay state.
//! Does not own: persistence, cache policy, catalog publication, or assurance promotion.
//! Boundary: replay is atomic; source follow-ups occur only in the pre-call-budgeted bootstrap API.

mod archive;
mod authentication;
mod bootstrap;
mod projection;
mod session;

use super::{
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    NnsCertifiedRegistryMutation, NnsCertifiedRegistryMutationKind, NnsRegistryHostError,
    validate_nns_certified_registry_delta_batch,
};
use std::collections::BTreeMap;
use thiserror::Error as ThisError;

pub use archive::{
    NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION, NnsAuthenticatedRegistryArchive,
    NnsCertifiedRegistryArchiveBatchDescriptor, NnsCertifiedRegistryArchiveBootstrapError,
    NnsCertifiedRegistryArchiveBootstrapRequest, NnsCertifiedRegistryArchiveError,
    NnsCertifiedRegistryArchiveLimits, NnsCertifiedRegistryArchiveManifest,
    NnsCertifiedRegistryArchiveManifestBuilder, NnsCertifiedRegistryArchivePublisher,
    NnsCertifiedRegistryArchiveStorageError, NnsCertifiedRegistryArchiveStorageLimits,
    bootstrap_nns_certified_registry_archive_async,
    bootstrap_nns_certified_registry_archive_with_source_async,
    load_nns_certified_registry_archive, nns_certified_registry_archive_manifest_path,
    nns_certified_registry_archive_refresh_lock_path,
    validate_nns_certified_registry_archive_manifest,
};
pub use authentication::{
    NnsAuthenticatedRegistryReplayBuilder, NnsAuthenticatedRegistryReplaySession,
};
pub use bootstrap::{
    NnsCertifiedRegistryBootstrapProbeOutcome, NnsCertifiedRegistryBootstrapProbeStatus,
    NnsCertifiedRegistryBootstrapRequest, bootstrap_nns_certified_registry_async,
    bootstrap_nns_certified_registry_with_source_async, probe_nns_certified_registry_async,
    probe_nns_certified_registry_with_source_async,
};
pub use projection::{
    NnsCertifiedSubnetCatalogAuthority, NnsCertifiedSubnetCatalogFreshness,
    NnsCertifiedSubnetCatalogProjectionRequest, NnsCertifiedSubnetCatalogVersionPolicy,
    NnsRegistrySubnetCatalogProjection, NnsRegistrySubnetCatalogProjectionError,
    project_nns_certified_subnet_catalog, project_nns_registry_subnet_catalog,
};
pub use session::{
    NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION, NnsRegistryReplaySession,
    NnsRegistryReplaySessionLimits,
};

///
/// NnsRegistryReplayLimits
///
/// Caller-selected payload ceilings for one published Registry replay state.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsRegistryReplayLimits {
    /// Maximum live Registry keys retained after any applied mutation.
    pub max_entries: usize,
    /// Maximum combined raw key and live value bytes in the resulting state.
    pub max_content_bytes: usize,
}

impl NnsRegistryReplayLimits {
    /// Create explicit replay-state ceilings without selecting hidden defaults.
    #[must_use]
    pub const fn new(max_entries: usize, max_content_bytes: usize) -> Self {
        Self {
            max_entries,
            max_content_bytes,
        }
    }
}

///
/// NnsRegistryReplayValue
///
/// Current value and last committed mutation evidence for one Registry key.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistryReplayValue {
    value: Vec<u8>,
    last_mutation_version: u64,
    timestamp_nanoseconds: u64,
}

impl NnsRegistryReplayValue {
    /// Return the current raw Registry value bytes.
    #[must_use]
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Return the Registry version that last mutated this key.
    #[must_use]
    pub const fn last_mutation_version(&self) -> u64 {
        self.last_mutation_version
    }

    /// Return the Registry-assigned timestamp of the last mutation.
    #[must_use]
    pub const fn timestamp_nanoseconds(&self) -> u64 {
        self.timestamp_nanoseconds
    }
}

///
/// NnsRegistryReplayState
///
/// Canonically ordered current Registry key state reconstructed from version zero.
/// This in-memory state is not authority evidence by itself. Callers must retain
/// and validate the certified batches that produced it.
///

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NnsRegistryReplayState {
    through_version: u64,
    content_bytes: usize,
    entries: BTreeMap<Vec<u8>, NnsRegistryReplayValue>,
}

impl NnsRegistryReplayState {
    /// Create an empty Registry state immediately before version one.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            through_version: 0,
            content_bytes: 0,
            entries: BTreeMap::new(),
        }
    }

    /// Return the last Registry version applied to this state.
    #[must_use]
    pub const fn through_version(&self) -> u64 {
        self.through_version
    }

    /// Return the number of currently present Registry keys.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Return combined raw key and current value bytes.
    #[must_use]
    pub const fn content_bytes(&self) -> usize {
        self.content_bytes
    }

    /// Return whether the current Registry state has no present keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up one current Registry value by raw key bytes.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&NnsRegistryReplayValue> {
        self.entries.get(key)
    }

    /// Iterate current Registry keys in canonical raw-byte order.
    pub fn entries(&self) -> impl Iterator<Item = (&[u8], &NnsRegistryReplayValue)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_slice(), value))
    }
}

///
/// NnsRegistryReplayProgress
///
/// Derived state and completion accounting after one atomic batch application.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistryReplayProgress {
    /// Registry version held before the batch was applied.
    pub previous_version: u64,
    /// Last Registry version held after the batch was applied.
    pub through_version: u64,
    /// Number of contiguous Registry versions applied from this batch.
    pub applied_version_count: usize,
    /// Number of committed mutations applied from this batch.
    pub applied_mutation_count: usize,
    /// Number of currently present Registry keys.
    pub entry_count: usize,
    /// Combined raw key and current value bytes.
    pub content_bytes: usize,
    /// Whether the state reached this batch's certified latest version.
    pub complete_at_certified_latest_version: bool,
}

///
/// NnsRegistryReplayError
///
/// Typed failures returned before an invalid or oversized replay state is published.
///

#[derive(Debug, ThisError)]
pub enum NnsRegistryReplayError {
    /// The supplied certified batch report failed its public contract.
    #[error(transparent)]
    InvalidBatch(#[from] NnsRegistryHostError),

    /// The batch does not continue immediately after the current replay state.
    #[error(
        "Registry replay state is at version {state_version}, but the batch starts after version {requested_version}"
    )]
    VersionMismatch {
        /// Version currently held by the replay state.
        state_version: u64,
        /// Version named by the batch request and report.
        requested_version: u64,
    },

    /// A completed exact-target replay session cannot accept another batch.
    #[error("Registry replay session already reached selected version {selected_version}")]
    SessionComplete {
        /// Exact Registry version selected from the session's first batch.
        selected_version: u64,
    },

    /// Archive extension was requested before the current segment reached its target.
    #[error(
        "Registry archive extension requires a complete segment: selected version {selected_version:?}, through version {through_version}"
    )]
    ArchiveExtensionRequiresCompleteSegment {
        /// Exact target selected by the current segment, when available.
        selected_version: Option<u64>,
        /// Last Registry version reconstructed by the current segment.
        through_version: u64,
    },

    /// A later batch no longer certifies the session's selected Registry version.
    #[error(
        "Registry replay selected version {selected_version}, but a later batch certifies only version {certified_latest_version}"
    )]
    CertifiedVersionRegressed {
        /// Exact Registry version selected from the first accepted batch.
        selected_version: u64,
        /// Latest Registry version certified by the later batch.
        certified_latest_version: u64,
    },

    /// Certified batches in one replay session disagree about the trusted root key.
    #[error(
        "Registry replay root-key digest changed from {expected_root_key_digest} to {actual_root_key_digest}"
    )]
    RootKeyDigestMismatch {
        /// Root-key digest retained from the first accepted batch.
        expected_root_key_digest: String,
        /// Root-key digest supplied by the later batch.
        actual_root_key_digest: String,
    },

    /// Accepted session evidence would exceed an explicit cumulative ceiling.
    #[error("Registry replay session {field} would be {actual}; caller maximum is {maximum}")]
    SessionLimitExceeded {
        /// Bounded cumulative replay-session resource.
        field: &'static str,
        /// Caller-selected cumulative ceiling.
        maximum: u64,
        /// Candidate cumulative amount.
        actual: u64,
    },

    /// The existing or candidate state exceeds a caller-selected payload ceiling.
    #[error("Registry replay {field} is {actual}; caller maximum is {maximum}")]
    LimitExceeded {
        /// Bounded replay-state resource.
        field: &'static str,
        /// Caller-selected ceiling.
        maximum: usize,
        /// Existing or candidate amount.
        actual: usize,
    },

    /// Validated report content could not be decoded defensively.
    #[error("validated Registry replay {field} is not lowercase hexadecimal")]
    InvalidHex {
        /// Report field that failed decoding.
        field: &'static str,
    },

    /// Internal byte accounting could not be represented consistently.
    #[error("Registry replay byte accounting overflowed or became inconsistent")]
    Accounting,

    /// A validated report could not be streamed into its evidence commitment.
    #[error("validated Registry replay evidence could not be encoded: {reason}")]
    EvidenceEncoding {
        /// Serialization failure returned while hashing the validated report.
        reason: String,
    },

    /// Type-level authentication was requested for incomplete replay evidence.
    #[error(
        "authenticated Registry replay requires a complete session: selected version {selected_version:?}, through version {through_version}"
    )]
    AuthenticationRequiresCompleteSession {
        /// Exact target selected from the first admitted report, when available.
        selected_version: Option<u64>,
        /// Last Registry version reconstructed by the session.
        through_version: u64,
    },
}

/// Apply exactly one validated certified Registry delta batch atomically.
///
/// The committed changelog has already passed Registry mutation checks. To
/// match the official Registry reconstruction path, insert, update, and upsert
/// rows all replace the current value; delete rows remove it. Recorded
/// preconditions are evidence attached to the committed transaction and are
/// not re-evaluated during replay.
pub fn apply_nns_certified_registry_delta_batch(
    state: &mut NnsRegistryReplayState,
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    report: &NnsCertifiedRegistryDeltaBatchReport,
    limits: NnsRegistryReplayLimits,
) -> Result<NnsRegistryReplayProgress, NnsRegistryReplayError> {
    validate_nns_certified_registry_delta_batch(request, report)?;
    apply_validated_batch_through(
        state,
        report,
        report.last_version.unwrap_or(report.requested_version),
        limits,
    )
}

pub(super) fn apply_validated_batch_through(
    state: &mut NnsRegistryReplayState,
    report: &NnsCertifiedRegistryDeltaBatchReport,
    selected_version: u64,
    limits: NnsRegistryReplayLimits,
) -> Result<NnsRegistryReplayProgress, NnsRegistryReplayError> {
    if state.through_version != report.requested_version {
        return Err(NnsRegistryReplayError::VersionMismatch {
            state_version: state.through_version,
            requested_version: report.requested_version,
        });
    }
    enforce_limits(state, limits)?;
    let (applied_version_count, applied_mutation_count) =
        validated_batch_prefix_counts(report, selected_version)?;

    let mut journal = ReplayJournal::new(state);
    let application = (|| {
        for version in &report.versions[..applied_version_count] {
            for mutation in &version.mutations {
                apply_committed_mutation(
                    state,
                    &mut journal,
                    version.version,
                    version.timestamp_nanoseconds,
                    mutation,
                    limits,
                )?;
            }
            state.through_version = version.version;
        }
        enforce_limits(state, limits)
    })();
    if let Err(error) = application {
        journal.rollback(state);
        return Err(error);
    }
    let progress = NnsRegistryReplayProgress {
        previous_version: journal.previous_version,
        through_version: state.through_version,
        applied_version_count,
        applied_mutation_count,
        entry_count: state.entries.len(),
        content_bytes: state.content_bytes,
        complete_at_certified_latest_version: state.through_version
            == report.certified_latest_version,
    };
    Ok(progress)
}

pub(super) fn validated_batch_prefix_counts(
    report: &NnsCertifiedRegistryDeltaBatchReport,
    selected_version: u64,
) -> Result<(usize, usize), NnsRegistryReplayError> {
    let version_count = report
        .versions
        .partition_point(|version| version.version <= selected_version);
    let mutation_count =
        report.versions[..version_count]
            .iter()
            .try_fold(0usize, |total, version| {
                total
                    .checked_add(version.mutations.len())
                    .ok_or(NnsRegistryReplayError::Accounting)
            })?;
    Ok((version_count, mutation_count))
}

fn apply_committed_mutation(
    state: &mut NnsRegistryReplayState,
    journal: &mut ReplayJournal,
    version: u64,
    timestamp_nanoseconds: u64,
    mutation: &NnsCertifiedRegistryMutation,
    limits: NnsRegistryReplayLimits,
) -> Result<(), NnsRegistryReplayError> {
    let key = decode_hex("mutation key", &mutation.key_hex)?;
    if mutation.mutation_kind == NnsCertifiedRegistryMutationKind::Delete {
        let prior_content_bytes = journal.remove_current(state, &key)?;
        state.content_bytes = state
            .content_bytes
            .checked_sub(prior_content_bytes)
            .ok_or(NnsRegistryReplayError::Accounting)?;
        return Ok(());
    }
    let value_hex = mutation
        .value_hex
        .as_deref()
        .ok_or(NnsRegistryReplayError::InvalidHex {
            field: "mutation value",
        })?;
    let value_bytes = value_hex.len() / 2;
    let prior_content_bytes = journal.remove_current(state, &key)?;
    let candidate_content_bytes = state
        .content_bytes
        .checked_sub(prior_content_bytes)
        .and_then(|bytes| bytes.checked_add(key.len()))
        .and_then(|bytes| bytes.checked_add(value_bytes))
        .ok_or(NnsRegistryReplayError::Accounting)?;
    enforce_limit(
        "content bytes",
        candidate_content_bytes,
        limits.max_content_bytes,
    )?;
    let candidate_entry_count = state
        .entries
        .len()
        .checked_add(1)
        .ok_or(NnsRegistryReplayError::Accounting)?;
    enforce_limit("entry count", candidate_entry_count, limits.max_entries)?;

    state.entries.insert(
        key,
        NnsRegistryReplayValue {
            value: decode_hex("mutation value", value_hex)?,
            last_mutation_version: version,
            timestamp_nanoseconds,
        },
    );
    state.content_bytes = candidate_content_bytes;
    Ok(())
}

struct ReplayJournal {
    previous_version: u64,
    previous_content_bytes: usize,
    entries: BTreeMap<Vec<u8>, Option<NnsRegistryReplayValue>>,
}

impl ReplayJournal {
    const fn new(state: &NnsRegistryReplayState) -> Self {
        Self {
            previous_version: state.through_version,
            previous_content_bytes: state.content_bytes,
            entries: BTreeMap::new(),
        }
    }

    fn remove_current(
        &mut self,
        state: &mut NnsRegistryReplayState,
        key: &[u8],
    ) -> Result<usize, NnsRegistryReplayError> {
        let current = state.entries.remove(key);
        let content_bytes = current.as_ref().map_or(Ok(0), |value| {
            key.len()
                .checked_add(value.value.len())
                .ok_or(NnsRegistryReplayError::Accounting)
        })?;
        self.entries.entry(key.to_vec()).or_insert(current);
        Ok(content_bytes)
    }

    fn rollback(self, state: &mut NnsRegistryReplayState) {
        for (key, original) in self.entries {
            state.entries.remove(&key);
            if let Some(value) = original {
                state.entries.insert(key, value);
            }
        }
        state.through_version = self.previous_version;
        state.content_bytes = self.previous_content_bytes;
    }
}

fn enforce_limits(
    state: &NnsRegistryReplayState,
    limits: NnsRegistryReplayLimits,
) -> Result<(), NnsRegistryReplayError> {
    enforce_limit("entry count", state.entries.len(), limits.max_entries)?;
    enforce_limit(
        "content bytes",
        state.content_bytes,
        limits.max_content_bytes,
    )
}

const fn enforce_limit(
    field: &'static str,
    actual: usize,
    maximum: usize,
) -> Result<(), NnsRegistryReplayError> {
    if actual > maximum {
        Err(NnsRegistryReplayError::LimitExceeded {
            field,
            maximum,
            actual,
        })
    } else {
        Ok(())
    }
}

fn decode_hex(field: &'static str, value: &str) -> Result<Vec<u8>, NnsRegistryReplayError> {
    crate::hex::decode_lowercase_hex(value).ok_or(NnsRegistryReplayError::InvalidHex { field })
}

#[cfg(test)]
mod tests;
