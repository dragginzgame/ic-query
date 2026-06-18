//! Module: sns::report::neurons_cache::collection::fetch::attempt
//!
//! Responsibility: write in-progress SNS neuron collection attempts.
//! Does not own: attempt model construction, cache publishing, page fetching, or rendering.
//! Boundary: records page and row progress after each fetched neuron page.

use super::state::SnsNeuronsCollectionState;
use crate::snapshot_cache::PagedCollectionPage;
use crate::sns::report::{
    SnsHostError, SnsNeuronsRefreshRequest,
    neurons_cache::attempt::{
        SnsNeuronsAttemptContext, SnsNeuronsAttemptProgress, write_running_sns_neurons_attempt,
    },
    source::{MainnetSns, SnsFetchRequest},
};
use std::path::Path;

pub(super) fn write_running_attempt(
    attempt_path: &Path,
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    state: &SnsNeuronsCollectionState,
    page: &PagedCollectionPage,
) -> Result<(), SnsHostError> {
    write_running_sns_neurons_attempt(
        SnsNeuronsAttemptContext {
            path: attempt_path,
            request,
            fetch_request,
            sns,
        },
        SnsNeuronsAttemptProgress::new(
            state.page_count(),
            state.row_count(),
            page.last_cursor_text.clone(),
        ),
    )
}
