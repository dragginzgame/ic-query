//! Module: sns::report::proposals_cache::storage::summary
//!
//! Responsibility: build proposal cache summaries and attempt status DTOs.
//! Does not own: cache path scanning, refresh orchestration, or text rendering.
//! Boundary: maps loaded proposal snapshots into public cache report models.

use super::{load::load_sns_proposals_cache_at, scan::collect_sns_proposals_cache_paths};
use crate::sns::report::proposals_cache::attempt::read_sns_proposals_attempt_status;
use crate::sns::report::{
    SnsHostError, SnsProposalsCacheSummary,
    proposals_cache::{model::SnsProposalsCache, paths::attempt_path_for_cache_path},
};
use std::path::{Path, PathBuf};

/// List summaries for all complete SNS proposal caches under one network.
pub(in crate::sns::report::proposals_cache) fn list_sns_proposals_cache_summaries(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<SnsProposalsCacheSummary>, SnsHostError> {
    collect_sns_proposals_cache_paths(icp_root, network)?
        .into_iter()
        .map(|path| {
            let cache = load_sns_proposals_cache_at(path.clone(), network)?;
            Ok(sns_proposals_cache_summary(path, cache))
        })
        .collect()
}

/// Build a public cache summary from one loaded complete proposal snapshot.
pub(in crate::sns::report::proposals_cache) fn sns_proposals_cache_summary(
    cache_path: PathBuf,
    cache: SnsProposalsCache,
) -> SnsProposalsCacheSummary {
    let attempt_path = attempt_path_for_cache_path(&cache_path);
    let latest_attempt = read_sns_proposals_attempt_status(&attempt_path);
    SnsProposalsCacheSummary {
        id: cache.metadata.id,
        name: cache.metadata.name,
        root_canister_id: cache.metadata.root_canister_id,
        governance_canister_id: cache.metadata.governance_canister_id,
        complete: cache.completeness.is_api_exhausted(),
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        refresh_attempt_path: attempt_path.display().to_string(),
        cache_path: cache_path.display().to_string(),
        latest_attempt,
    }
}
