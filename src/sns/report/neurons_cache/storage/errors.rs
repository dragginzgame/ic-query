use crate::{cache_file::LoadJsonCacheErrorMapper, sns::report::SnsHostError};
use std::path::PathBuf;

pub(super) struct SnsNeuronsCacheErrors;

impl LoadJsonCacheErrorMapper for SnsNeuronsCacheErrors {
    type Error = SnsHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        SnsHostError::MissingNeuronsCache { path }
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

pub(super) fn incomplete_cache_error(page_count: u32, row_count: usize) -> SnsHostError {
    SnsHostError::IncompleteRefresh {
        pages_fetched: page_count,
        rows_fetched: row_count,
        reason: "cached SNS neurons snapshot is not complete".to_string(),
    }
}

pub(super) const fn missing_id_error(id: usize, root: PathBuf) -> SnsHostError {
    SnsHostError::MissingNeuronsCacheForId { id, root }
}

pub(super) fn invalid_lookup_error(input: &str) -> SnsHostError {
    SnsHostError::InvalidLookup {
        input: input.to_string(),
    }
}
