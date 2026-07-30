//! Module: nns::proposals::report::cache::attempt
//!
//! Responsibility: read and write NNS proposal refresh-attempt metadata.
//! Does not own: live proposal paging, cache publication, or text rendering.
//! Boundary: persists refresh lifecycle status for cache status reports.

use super::{NNS_PROPOSAL_CACHE_COMPONENT, model::NnsProposalRefreshAttempt};
use crate::{
    HostCacheError,
    nns::{
        NnsGovernanceRefreshAttemptStatus, NnsGovernanceRefreshRequest,
        governance::{
            NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS, governance_refresh_attempt_status,
            governance_refresh_progress, mainnet_governance_cache_metadata,
            validate_governance_cache_metadata,
        },
        proposals::report::NnsProposalHostError,
    },
    snapshot_cache::{
        SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt,
        SnapshotRefreshAttemptReadError, SnapshotRefreshProgress, current_attempt_timestamp,
        read_snapshot_refresh_attempt_strict, validate_snapshot_refresh_attempt,
        write_snapshot_refresh_attempt,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use std::path::Path;

pub(super) fn read_attempt_status(path: &Path) -> Option<NnsGovernanceRefreshAttemptStatus> {
    let attempt =
        read_snapshot_refresh_attempt_strict(path, NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS)
            .ok()??;
    validate_nns_attempt(path, MAINNET_NETWORK, &attempt).ok()?;
    Some(governance_refresh_attempt_status(attempt))
}

pub(super) fn read_attempt_status_strict(
    path: &Path,
    expected_network: &str,
) -> Result<Option<NnsGovernanceRefreshAttemptStatus>, NnsProposalHostError> {
    read_snapshot_refresh_attempt_strict::<NnsProposalRefreshAttempt>(
        path,
        NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS,
    )
    .map_err(|err| match err {
        SnapshotRefreshAttemptReadError::Read { path, source } => NnsProposalHostError::Cache(
            HostCacheError::read_cache(NNS_PROPOSAL_CACHE_COMPONENT, path, source),
        ),
        SnapshotRefreshAttemptReadError::Parse { path, source } => NnsProposalHostError::Cache(
            HostCacheError::parse_cache(NNS_PROPOSAL_CACHE_COMPONENT, path, source),
        ),
        SnapshotRefreshAttemptReadError::Invalid { path, reason } => {
            NnsProposalHostError::InvalidRefreshAttempt { path, reason }
        }
    })?
    .map(|attempt| {
        validate_nns_attempt(path, expected_network, &attempt)?;
        Ok(governance_refresh_attempt_status(attempt))
    })
    .transpose()
}

pub(super) fn write_starting_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(
        path,
        request,
        "running",
        SnapshotRefreshProgress::default(),
        None,
    )
}

pub(super) fn write_running_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    progress: SnapshotRefreshProgress,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(path, request, "running", progress, None)
}

pub(super) fn write_complete_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    progress: SnapshotRefreshProgress,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(path, request, "complete", progress, None)
}

pub(super) fn write_failed_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    err: &NnsProposalHostError,
) {
    let latest = read_snapshot_refresh_attempt_strict(path, NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS)
        .ok()
        .flatten();
    let latest =
        latest.filter(|attempt| validate_nns_attempt(path, &request.network, attempt).is_ok());
    let progress = latest.map(governance_refresh_progress).unwrap_or_default();
    let _ = write_attempt_status(path, request, "failed", progress, Some(err.to_string()));
}

fn validate_nns_attempt(
    path: &Path,
    expected_network: &str,
    attempt: &NnsProposalRefreshAttempt,
) -> Result<(), NnsProposalHostError> {
    let invalid = |reason| NnsProposalHostError::InvalidRefreshAttempt {
        path: path.to_path_buf(),
        reason,
    };
    validate_snapshot_refresh_attempt(attempt, expected_network).map_err(invalid)?;
    validate_governance_cache_metadata(&attempt.metadata).map_err(invalid)
}

fn write_attempt_status(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    status: &'static str,
    progress: SnapshotRefreshProgress,
    last_error: Option<String>,
) -> Result<(), NnsProposalHostError> {
    let timestamp = format_utc_timestamp_secs(request.now_unix_secs);
    let attempt: NnsProposalRefreshAttempt = SnapshotRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        started_at: timestamp.clone(),
        updated_at: current_attempt_timestamp(&timestamp),
        metadata: mainnet_governance_cache_metadata(),
        status: status.to_string(),
        page_size: request.page_size,
        pages_fetched: progress.pages_fetched,
        rows_fetched: progress.rows_fetched,
        last_cursor: progress.last_cursor,
        last_error,
    };
    write_snapshot_refresh_attempt(
        path,
        &attempt,
        |path, source| {
            NnsProposalHostError::Cache(HostCacheError::serialize_cache(
                NNS_PROPOSAL_CACHE_COMPONENT,
                path,
                source,
            ))
        },
        |error| {
            NnsProposalHostError::Cache(HostCacheError::operation(
                NNS_PROPOSAL_CACHE_COMPONENT,
                error,
            ))
        },
    )
}
