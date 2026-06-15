use super::{
    model::{SnsNeuronsAttemptParts, SnsNeuronsRefreshAttempt, attempt_from_parts},
    read::read_sns_neurons_attempt,
};
use crate::sns::report::{
    SnsHostError, SnsNeuronsRefreshRequest,
    source::{MainnetSns, SnsFetchRequest},
};
use std::path::Path;

pub(in crate::sns::report::neurons_cache) fn failed_attempt_from_latest_progress(
    attempt_path: &Path,
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    err: &SnsHostError,
) -> SnsNeuronsRefreshAttempt {
    let latest = read_sns_neurons_attempt(attempt_path);
    let pages_fetched = latest.as_ref().map_or(0, |attempt| attempt.pages_fetched);
    let rows_fetched = latest.as_ref().map_or(0, |attempt| attempt.rows_fetched);
    let last_cursor = latest.and_then(|attempt| attempt.last_cursor);
    attempt_from_parts(SnsNeuronsAttemptParts {
        request,
        fetch_request,
        sns,
        status: "failed",
        pages_fetched,
        rows_fetched,
        last_cursor,
        last_error: Some(err.to_string()),
    })
}
