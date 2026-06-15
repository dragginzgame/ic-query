use super::state::{SnsNeuronsCollectionPage, SnsNeuronsCollectionState};
use crate::sns::report::{
    SnsHostError, SnsNeuronsRefreshRequest,
    neurons_cache::attempt::{
        SnsNeuronsAttemptParts, attempt_from_parts, write_sns_neurons_attempt,
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
    page: &SnsNeuronsCollectionPage,
) -> Result<(), SnsHostError> {
    write_sns_neurons_attempt(
        attempt_path,
        &attempt_from_parts(SnsNeuronsAttemptParts {
            request,
            fetch_request,
            sns,
            status: "running",
            pages_fetched: state.page_count(),
            rows_fetched: state.row_count(),
            last_cursor: page.last_cursor_text.clone(),
            last_error: None,
        }),
    )
}
