//! Module: nns::governance::collection
//!
//! Responsibility: shared host contracts and attempt IO for NNS Governance collections.
//! Does not own: proposal or neuron paging, cache paths, or report rendering.
//! Boundary: centralizes shared DTOs plus attempt construction and validation.

use crate::{
    HostCacheError,
    cache::CacheRefreshAttemptStatus,
    nns::MAINNET_GOVERNANCE_CANISTER_ID,
    snapshot_cache::{
        SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt,
        SnapshotRefreshAttemptReadError, SnapshotRefreshProgress, current_attempt_timestamp,
        read_snapshot_refresh_attempt_strict, validate_snapshot_refresh_attempt,
        write_snapshot_refresh_attempt,
    },
    subnet_catalog::format_utc_timestamp_secs,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::{Path, PathBuf};

/// Refresh-attempt fields owned by NNS Governance collection metadata.
pub(in crate::nns) const NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS: &[&str] =
    &["governance_canister_id"];

///
/// NnsGovernanceRefreshRequest
///
/// Shared request for one complete NNS Governance collection refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsGovernanceRefreshRequest {
    /// Root directory containing shared caches.
    pub cache_root: PathBuf,
    /// Requested network identity.
    pub network: String,
    /// Replica endpoint used for every page.
    pub source_endpoint: String,
    /// Caller-provided collection time in Unix seconds.
    pub now_unix_secs: u64,
    /// Governance page size.
    pub page_size: u32,
    /// Optional diagnostic page cap.
    pub max_pages: Option<u32>,
}

impl NnsGovernanceRefreshRequest {
    /// Construct a complete NNS Governance collection refresh request.
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        page_size: u32,
    ) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            page_size,
            max_pages: None,
        }
    }

    /// Stop diagnostically before fetching more than the given pages.
    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: Option<u32>) -> Self {
        self.max_pages = max_pages;
        self
    }
}

///
/// NnsGovernanceCacheRequest
///
/// Shared local cache scope for complete NNS Governance collections.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsGovernanceCacheRequest {
    /// Root directory containing shared caches.
    pub cache_root: PathBuf,
    /// Requested network identity.
    pub network: String,
}

impl NnsGovernanceCacheRequest {
    /// Construct a local NNS Governance cache request.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }

    /// Return the shared cache root.
    #[must_use]
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }
}

///
/// NnsGovernanceRefreshAttemptStatus
///
/// Serializable lifecycle evidence for an NNS Governance collection refresh.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsGovernanceRefreshAttemptStatus {
    /// Attempt lifecycle status.
    pub status: CacheRefreshAttemptStatus,
    /// Attempt start timestamp.
    pub started_at: String,
    /// Latest attempt update timestamp.
    pub updated_at: String,
    /// Requested page size.
    pub page_size: u32,
    /// Successfully retained pages.
    pub pages_fetched: u32,
    /// Successfully retained rows.
    pub rows_fetched: usize,
    /// Latest exclusive cursor.
    pub last_cursor: Option<String>,
    /// Latest terminal error.
    pub last_error: Option<String>,
}

///
/// NnsGovernanceCacheMetadata
///
/// Shared flattened provenance for an NNS Governance snapshot or attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::nns) struct NnsGovernanceCacheMetadata {
    /// NNS Governance canister principal.
    pub(in crate::nns) governance_canister_id: String,
}

///
/// NnsGovernanceAttemptReadError
///
/// Shared internal failure from reading or validating Governance attempt evidence.
///

#[derive(Debug)]
pub(in crate::nns) enum NnsGovernanceAttemptReadError {
    /// Cache-file read or JSON parsing failed.
    Cache(HostCacheError),
    /// The sidecar did not describe the requested Governance collection.
    Invalid {
        /// Attempt sidecar path.
        path: PathBuf,
        /// Deterministic validation failure.
        reason: String,
    },
}

/// Construct canonical mainnet NNS Governance cache metadata.
#[must_use]
pub(in crate::nns) fn mainnet_governance_cache_metadata() -> NnsGovernanceCacheMetadata {
    NnsGovernanceCacheMetadata {
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
    }
}

/// Validate the Governance canister identity in shared cache metadata.
pub(in crate::nns) fn validate_governance_cache_metadata(
    metadata: &NnsGovernanceCacheMetadata,
) -> Result<(), String> {
    if metadata.governance_canister_id == MAINNET_GOVERNANCE_CANISTER_ID {
        return Ok(());
    }
    Err(format!(
        "governance_canister_id is {}, expected {MAINNET_GOVERNANCE_CANISTER_ID}",
        metadata.governance_canister_id
    ))
}

/// Project a stored refresh attempt into its report-safe lifecycle fields.
#[must_use]
pub(in crate::nns) fn governance_refresh_attempt_status<Metadata>(
    attempt: SnapshotRefreshAttempt<Metadata>,
    status: CacheRefreshAttemptStatus,
) -> NnsGovernanceRefreshAttemptStatus {
    NnsGovernanceRefreshAttemptStatus {
        status,
        started_at: attempt.started_at,
        updated_at: attempt.updated_at,
        page_size: attempt.page_size,
        pages_fetched: attempt.pages_fetched,
        rows_fetched: attempt.rows_fetched,
        last_cursor: attempt.last_cursor,
        last_error: attempt.last_error,
    }
}

/// Recover collection progress from one stored NNS Governance attempt.
#[must_use]
pub(in crate::nns) fn governance_refresh_progress<Metadata>(
    attempt: SnapshotRefreshAttempt<Metadata>,
) -> SnapshotRefreshProgress {
    SnapshotRefreshProgress::new(
        attempt.pages_fetched,
        attempt.rows_fetched,
        attempt.last_cursor,
    )
}

/// Strictly read and validate one NNS Governance refresh-attempt sidecar.
pub(in crate::nns) fn read_governance_refresh_attempt(
    cache_root: &Path,
    path: &Path,
    expected_network: &str,
    cache_component: &'static str,
) -> Result<
    Option<(
        SnapshotRefreshAttempt<NnsGovernanceCacheMetadata>,
        CacheRefreshAttemptStatus,
    )>,
    NnsGovernanceAttemptReadError,
> {
    let attempt = read_snapshot_refresh_attempt_strict::<
        SnapshotRefreshAttempt<NnsGovernanceCacheMetadata>,
    >(cache_root, path, NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS)
    .map_err(|error| match error {
        SnapshotRefreshAttemptReadError::Operation(source) => {
            NnsGovernanceAttemptReadError::Cache(HostCacheError::operation(cache_component, source))
        }
        SnapshotRefreshAttemptReadError::Parse { path, source } => {
            NnsGovernanceAttemptReadError::Cache(HostCacheError::parse_cache(
                cache_component,
                path,
                source,
            ))
        }
        SnapshotRefreshAttemptReadError::Invalid { path, reason } => {
            NnsGovernanceAttemptReadError::Invalid { path, reason }
        }
    })?;
    attempt
        .map(|attempt| {
            let invalid = |reason| NnsGovernanceAttemptReadError::Invalid {
                path: path.to_path_buf(),
                reason,
            };
            let status =
                validate_snapshot_refresh_attempt(&attempt, expected_network).map_err(invalid)?;
            validate_governance_cache_metadata(&attempt.metadata).map_err(invalid)?;
            Ok((attempt, status))
        })
        .transpose()
}

/// Strictly read one report-safe NNS Governance refresh-attempt status.
pub(in crate::nns) fn read_governance_refresh_attempt_status(
    cache_root: &Path,
    path: &Path,
    expected_network: &str,
    cache_component: &'static str,
) -> Result<Option<NnsGovernanceRefreshAttemptStatus>, NnsGovernanceAttemptReadError> {
    read_governance_refresh_attempt(cache_root, path, expected_network, cache_component).map(
        |attempt| {
            attempt.map(|(attempt, status)| governance_refresh_attempt_status(attempt, status))
        },
    )
}

/// Construct and write one validated-shape NNS Governance refresh-attempt sidecar.
fn write_governance_refresh_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    cache_component: &'static str,
    status: CacheRefreshAttemptStatus,
    progress: SnapshotRefreshProgress,
    last_error: Option<String>,
) -> Result<(), HostCacheError> {
    let started_at = format_utc_timestamp_secs(request.now_unix_secs);
    let attempt = SnapshotRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        started_at: started_at.clone(),
        updated_at: current_attempt_timestamp(&started_at),
        metadata: mainnet_governance_cache_metadata(),
        status: status.to_string(),
        page_size: request.page_size,
        pages_fetched: progress.pages_fetched,
        rows_fetched: progress.rows_fetched,
        last_cursor: progress.last_cursor,
        last_error,
    };
    write_snapshot_refresh_attempt(
        &request.cache_root,
        path,
        &attempt,
        |path, source| HostCacheError::serialize_cache(cache_component, path, source),
        |error| HostCacheError::operation(cache_component, error),
    )
}

/// Write the initial running state for an NNS Governance refresh.
pub(in crate::nns) fn write_starting_governance_refresh_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    cache_component: &'static str,
) -> Result<(), HostCacheError> {
    write_governance_refresh_attempt(
        path,
        request,
        cache_component,
        CacheRefreshAttemptStatus::Running,
        SnapshotRefreshProgress::default(),
        None,
    )
}

/// Write the latest retained progress for an NNS Governance refresh.
pub(in crate::nns) fn write_running_governance_refresh_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    cache_component: &'static str,
    progress: SnapshotRefreshProgress,
) -> Result<(), HostCacheError> {
    write_governance_refresh_attempt(
        path,
        request,
        cache_component,
        CacheRefreshAttemptStatus::Running,
        progress,
        None,
    )
}

/// Write the terminal complete state for an NNS Governance refresh.
pub(in crate::nns) fn write_complete_governance_refresh_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    cache_component: &'static str,
    progress: SnapshotRefreshProgress,
) -> Result<(), HostCacheError> {
    write_governance_refresh_attempt(
        path,
        request,
        cache_component,
        CacheRefreshAttemptStatus::Complete,
        progress,
        None,
    )
}

/// Preserve valid progress and write failed NNS Governance refresh evidence.
pub(in crate::nns) fn write_failed_governance_refresh_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    cache_component: &'static str,
    last_error: String,
) -> Result<(), HostCacheError> {
    let progress = read_governance_refresh_attempt(
        &request.cache_root,
        path,
        &request.network,
        cache_component,
    )
    .ok()
    .flatten()
    .map(|(attempt, _status)| governance_refresh_progress(attempt))
    .unwrap_or_default();
    write_governance_refresh_attempt(
        path,
        request,
        cache_component,
        CacheRefreshAttemptStatus::Failed,
        progress,
        Some(last_error),
    )
}
