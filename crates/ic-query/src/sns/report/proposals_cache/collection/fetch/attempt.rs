//! Module: sns::report::proposals_cache::collection::fetch::attempt
//!
//! Responsibility: persist running proposal refresh progress.
//! Does not own: proposal page fetching, collection state, or cache publication.
//! Boundary: adapts collection state into refresh-attempt writer inputs.

use super::state::SnsProposalsCollectionState;
use crate::snapshot_cache::{PagedCollectionPage, SnapshotRefreshProgress};
use crate::sns::report::{
    SnsHostError, SnsProposalsRefreshRequest,
    cache_attempt::{SnsRefreshAttemptContext, write_running_sns_refresh_attempt},
    source::{MainnetSns, SnsSourceRequest},
};
use std::path::Path;

pub(super) fn write_running_attempt(
    attempt_path: &Path,
    request: &SnsProposalsRefreshRequest,
    fetch_request: &SnsSourceRequest,
    sns: &MainnetSns,
    state: &SnsProposalsCollectionState,
    page: &PagedCollectionPage,
) -> Result<(), SnsHostError> {
    write_running_sns_refresh_attempt(
        SnsRefreshAttemptContext {
            path: attempt_path,
            request,
            fetch_request,
            sns,
        },
        SnapshotRefreshProgress::new(
            state.page_count(),
            state.row_count(),
            page.last_cursor_text.clone(),
        ),
    )
}
