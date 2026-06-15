use super::super::errors::sns_cache_file_error;
use super::model::SnsNeuronsRefreshAttempt;
use crate::{cache_file::write_text_atomically, sns::report::SnsHostError};
use std::path::Path;

pub(in crate::sns::report::neurons_cache) fn write_sns_neurons_attempt(
    path: &Path,
    attempt: &SnsNeuronsRefreshAttempt,
) -> Result<(), SnsHostError> {
    let data =
        serde_json::to_string_pretty(attempt).map_err(|source| SnsHostError::SerializeCache {
            path: path.to_path_buf(),
            source,
        })?;
    write_text_atomically(path, &data).map_err(sns_cache_file_error)
}
