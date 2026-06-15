use super::super::errors::sns_cache_file_error;
use super::model::SnsNeuronsRefreshAttempt;
use crate::{snapshot_cache::write_snapshot_refresh_attempt, sns::report::SnsHostError};
use std::path::Path;

pub(in crate::sns::report::neurons_cache) fn write_sns_neurons_attempt(
    path: &Path,
    attempt: &SnsNeuronsRefreshAttempt,
) -> Result<(), SnsHostError> {
    write_snapshot_refresh_attempt(
        path,
        attempt,
        |path, source| SnsHostError::SerializeCache { path, source },
        sns_cache_file_error,
    )
}
