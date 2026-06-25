//! Module: sns::report::proposals_cache::errors
//!
//! Responsibility: proposal cache JSON error mapping.
//! Does not own: cache IO, refresh orchestration, or report rendering.
//! Boundary: maps proposal cache JSON failures to SNS host errors.

use crate::{cache_file::LoadJsonCacheErrorMapper, sns::report::SnsHostError};
use std::path::PathBuf;

///
/// SnsProposalsCacheErrors
///
/// Error mapper used when loading proposal snapshot JSON files.
///

pub(super) struct SnsProposalsCacheErrors;

impl LoadJsonCacheErrorMapper for SnsProposalsCacheErrors {
    type Error = SnsHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        SnsHostError::MissingProposalsCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: std::io::Error) -> Self::Error {
        SnsHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        SnsHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        SnsHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        SnsHostError::CacheNetworkMismatch { requested, actual }
    }
}
