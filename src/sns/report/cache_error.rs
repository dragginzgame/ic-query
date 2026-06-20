//! Module: sns::report::cache_error
//!
//! Responsibility: shared SNS cache-file error formatting.
//! Does not own: cache reading, refresh logic, or report-specific cache errors.
//! Boundary: maps generic cache-file failures into SNS host errors.

use crate::{cache_file::CacheFileError, sns::report::SnsHostError};

pub(super) fn sns_cache_file_error(err: CacheFileError) -> SnsHostError {
    SnsHostError::Cache(err.to_string())
}
