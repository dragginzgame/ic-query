use super::model::SnsNeuronsRefreshAttempt;
use crate::{
    snapshot_cache::read_snapshot_refresh_attempt, sns::report::SnsNeuronsRefreshAttemptStatus,
};
use std::path::Path;

pub(in crate::sns::report::neurons_cache::attempt) fn read_sns_neurons_attempt(
    path: &Path,
) -> Option<SnsNeuronsRefreshAttempt> {
    read_snapshot_refresh_attempt(path)
}

pub(in crate::sns::report::neurons_cache) fn read_sns_neurons_attempt_status(
    path: &Path,
) -> Option<SnsNeuronsRefreshAttemptStatus> {
    read_sns_neurons_attempt(path).map(sns_neurons_attempt_status)
}

fn sns_neurons_attempt_status(attempt: SnsNeuronsRefreshAttempt) -> SnsNeuronsRefreshAttemptStatus {
    SnsNeuronsRefreshAttemptStatus {
        status: attempt.status,
        started_at: attempt.started_at,
        updated_at: attempt.updated_at,
        page_size: attempt.page_size,
        pages_fetched: attempt.pages_fetched,
        rows_fetched: attempt.rows_fetched,
        last_cursor: attempt.last_cursor,
        last_error: attempt.last_error,
    }
}
