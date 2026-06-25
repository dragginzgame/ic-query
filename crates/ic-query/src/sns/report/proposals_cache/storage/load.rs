//! Module: sns::report::proposals_cache::storage::load
//!
//! Responsibility: load complete SNS proposal cache snapshots.
//! Does not own: cache path scanning, status summaries, or refresh orchestration.
//! Boundary: maps snapshot JSON loading errors into SNS host errors.

use crate::sns::report::{
    SnsHostError,
    cache_storage::load_sns_complete_cache,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION, errors::SnsProposalsCacheErrors,
        model::SnsProposalsCache, paths::sns_proposals_cache_key_for_cache_path,
    },
};
use std::path::PathBuf;

/// Load one complete SNS proposal snapshot from a concrete cache path.
pub(in crate::sns::report::proposals_cache) fn load_sns_proposals_cache_at(
    cache_path: PathBuf,
    network: &str,
) -> Result<SnsProposalsCache, SnsHostError> {
    let key = sns_proposals_cache_key_for_cache_path(network, &cache_path);
    load_sns_complete_cache(
        cache_path,
        network,
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        &key,
        SnsProposalsCacheErrors,
        |completeness| SnsHostError::IncompleteRefresh {
            pages_fetched: completeness.page_count,
            rows_fetched: completeness.row_count,
            reason: "cached SNS proposals snapshot is not complete".to_string(),
        },
    )
}
