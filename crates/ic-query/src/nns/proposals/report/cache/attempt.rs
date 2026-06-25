//! Module: nns::proposals::report::cache::attempt
//!
//! Responsibility: read and write NNS proposal refresh-attempt metadata.
//! Does not own: live proposal paging, cache publication, or text rendering.
//! Boundary: persists refresh lifecycle status for cache status reports.

use super::model::{
    NnsProposalRefreshAttempt, NnsProposalRefreshAttemptMetadata, NnsProposalRefreshAttemptStatus,
    NnsProposalRefreshRequest,
};
use crate::{
    nns::proposals::report::{MAINNET_GOVERNANCE_CANISTER_ID, NnsProposalHostError},
    snapshot_cache::{
        SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt,
        read_snapshot_refresh_attempt, write_snapshot_refresh_attempt,
    },
    subnet_catalog::format_utc_timestamp_secs,
};
use std::path::Path;

///
/// NnsProposalAttemptProgress
///
/// In-progress NNS proposal refresh page counters.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NnsProposalAttemptProgress {
    pub(super) pages_fetched: u32,
    pub(super) rows_fetched: usize,
    pub(super) last_cursor: Option<String>,
}

impl NnsProposalAttemptProgress {
    pub(super) const fn new(
        pages_fetched: u32,
        rows_fetched: usize,
        last_cursor: Option<String>,
    ) -> Self {
        Self {
            pages_fetched,
            rows_fetched,
            last_cursor,
        }
    }

    pub(super) const fn starting() -> Self {
        Self {
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
        }
    }
}

pub(super) fn read_attempt_status(path: &Path) -> Option<NnsProposalRefreshAttemptStatus> {
    let attempt = read_snapshot_refresh_attempt::<NnsProposalRefreshAttempt>(path)?;
    Some(NnsProposalRefreshAttemptStatus::from(attempt))
}

pub(super) fn write_starting_attempt(
    path: &Path,
    request: &NnsProposalRefreshRequest,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(
        path,
        request,
        "running",
        NnsProposalAttemptProgress::starting(),
        None,
    )
}

pub(super) fn write_running_attempt(
    path: &Path,
    request: &NnsProposalRefreshRequest,
    progress: NnsProposalAttemptProgress,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(path, request, "running", progress, None)
}

pub(super) fn write_complete_attempt(
    path: &Path,
    request: &NnsProposalRefreshRequest,
    progress: NnsProposalAttemptProgress,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(path, request, "complete", progress, None)
}

pub(super) fn write_failed_attempt(
    path: &Path,
    request: &NnsProposalRefreshRequest,
    err: &NnsProposalHostError,
) {
    let _ = write_attempt_status(
        path,
        request,
        "failed",
        NnsProposalAttemptProgress::starting(),
        Some(err.to_string()),
    );
}

fn write_attempt_status(
    path: &Path,
    request: &NnsProposalRefreshRequest,
    status: &'static str,
    progress: NnsProposalAttemptProgress,
    last_error: Option<String>,
) -> Result<(), NnsProposalHostError> {
    let timestamp = format_utc_timestamp_secs(request.now_unix_secs);
    let attempt: NnsProposalRefreshAttempt =
        SnapshotRefreshAttempt::<NnsProposalRefreshAttemptMetadata> {
            schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
            network: request.network.clone(),
            source_endpoint: request.source_endpoint.clone(),
            started_at: timestamp.clone(),
            updated_at: timestamp,
            metadata: NnsProposalRefreshAttemptMetadata {
                governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
            },
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
        |path, source| NnsProposalHostError::SerializeCache { path, source },
        NnsProposalHostError::Cache,
    )
}
