//! Module: sns::report::proposals_cache::collection::fetch::attempt
//!
//! Responsibility: persist running proposal refresh progress.
//! Does not own: proposal page fetching, collection state, or cache publication.
//! Boundary: adapts collection state into refresh-attempt writer inputs.

use super::state::SnsProposalsCollectionState;
use crate::snapshot_cache::PagedCollectionPage;
use crate::sns::report::{
    SnsHostError, SnsProposalsRefreshRequest,
    proposals_cache::attempt::{
        SnsProposalsAttemptContext, SnsProposalsAttemptProgress,
        write_running_attempt as write_running_proposals_attempt,
    },
    source::{MainnetSns, SnsFetchRequest},
};
use std::path::Path;

pub(super) fn write_running_attempt(
    attempt_path: &Path,
    request: &SnsProposalsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    state: &SnsProposalsCollectionState,
    page: &PagedCollectionPage,
) -> Result<(), SnsHostError> {
    write_running_proposals_attempt(
        SnsProposalsAttemptContext {
            path: attempt_path,
            request,
            fetch_request,
            sns,
        },
        SnsProposalsAttemptProgress::new(
            state.page_count(),
            state.row_count(),
            page.last_cursor_text.clone(),
        ),
    )
}
