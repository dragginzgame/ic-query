//! Module: sns::report::proposals_cache::attempt
//!
//! Responsibility: proposal refresh-attempt status construction and writes.
//! Does not own: cache publishing, page fetching, or cache status reporting.
//! Boundary: serializes running, complete, and failed proposal refresh attempts.

use super::{
    errors::sns_cache_file_error,
    model::{SnsProposalsRefreshAttempt, SnsProposalsRefreshAttemptMetadata},
};
use crate::{
    snapshot_cache::{SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, write_snapshot_refresh_attempt},
    sns::report::{
        SnsHostError, SnsProposalsRefreshRequest,
        source::{MainnetSns, SnsFetchRequest},
    },
};
use std::path::Path;

///
/// SnsProposalsAttemptContext
///
/// Shared context required to write one proposal refresh-attempt file.
///

#[derive(Clone, Copy)]
pub(super) struct SnsProposalsAttemptContext<'a> {
    pub(super) path: &'a Path,
    pub(super) request: &'a SnsProposalsRefreshRequest,
    pub(super) fetch_request: &'a SnsFetchRequest,
    pub(super) sns: &'a MainnetSns,
}

///
/// SnsProposalsAttemptProgress
///
/// Proposal refresh progress persisted into refresh-attempt metadata.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsProposalsAttemptProgress {
    pub(super) pages_fetched: u32,
    pub(super) rows_fetched: usize,
    pub(super) last_cursor: Option<String>,
}

impl SnsProposalsAttemptProgress {
    /// Build refresh-attempt progress from current paging counters.
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

    const fn starting() -> Self {
        Self {
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
        }
    }
}

/// Write the initial running proposal refresh-attempt file.
pub(super) fn write_starting_attempt(
    context: SnsProposalsAttemptContext<'_>,
) -> Result<(), SnsHostError> {
    write_attempt_status(
        context,
        "running",
        SnsProposalsAttemptProgress::starting(),
        None,
    )
}

/// Write updated running proposal refresh progress.
pub(super) fn write_running_attempt(
    context: SnsProposalsAttemptContext<'_>,
    progress: SnsProposalsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_attempt_status(context, "running", progress, None)
}

/// Write successful proposal refresh completion metadata.
pub(super) fn write_complete_attempt(
    context: SnsProposalsAttemptContext<'_>,
    progress: SnsProposalsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_attempt_status(context, "complete", progress, None)
}

/// Best-effort write of failed proposal refresh-attempt metadata.
pub(super) fn write_failed_attempt(context: SnsProposalsAttemptContext<'_>, err: &SnsHostError) {
    let _ = write_attempt_status(
        context,
        "failed",
        SnsProposalsAttemptProgress::starting(),
        Some(err.to_string()),
    );
}

fn write_attempt_status(
    context: SnsProposalsAttemptContext<'_>,
    status: &'static str,
    progress: SnsProposalsAttemptProgress,
    last_error: Option<String>,
) -> Result<(), SnsHostError> {
    let attempt = SnsProposalsRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: context.request.network.clone(),
        source_endpoint: context.request.source_endpoint.clone(),
        started_at: context.fetch_request.fetched_at.clone(),
        updated_at: context.fetch_request.fetched_at.clone(),
        metadata: SnsProposalsRefreshAttemptMetadata {
            root_canister_id: context.sns.root_canister_id.clone(),
            governance_canister_id: context.sns.governance_canister_id.clone(),
        },
        status: status.to_string(),
        page_size: context.request.page_size,
        pages_fetched: progress.pages_fetched,
        rows_fetched: progress.rows_fetched,
        last_cursor: progress.last_cursor,
        last_error,
    };
    write_snapshot_refresh_attempt(
        context.path,
        &attempt,
        |path, source| SnsHostError::SerializeCache { path, source },
        sns_cache_file_error,
    )
}
