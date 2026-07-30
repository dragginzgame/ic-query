//! Module: sns::report::proposals_cache::storage::summary
//!
//! Responsibility: build proposal cache summaries and attempt status DTOs.
//! Does not own: cache path scanning, refresh orchestration, or text rendering.
//! Boundary: maps loaded proposal snapshots into public cache report models.

use super::{load::load_sns_proposals_cache_at, scan::collect_sns_proposals_cache_paths};
use crate::sns::report::{
    SnsCacheSummary, SnsHostError, project_sns_cache_summary,
    proposals_cache::{model::SnsProposalsCache, paths::attempt_path_for_cache_path},
};
use std::path::{Path, PathBuf};

/// List summaries for all complete SNS proposal caches under one network.
pub(in crate::sns::report::proposals_cache) fn list_sns_proposals_cache_summaries(
    cache_root: &Path,
    network: &str,
) -> Result<Vec<SnsCacheSummary>, SnsHostError> {
    collect_sns_proposals_cache_paths(cache_root, network)?
        .into_iter()
        .map(|path| Ok(load_sns_proposals_cache_summary_at(path, network)))
        .collect()
}

pub(in crate::sns::report::proposals_cache) fn load_sns_proposals_cache_summary_at(
    cache_path: PathBuf,
    network: &str,
) -> SnsCacheSummary {
    match load_sns_proposals_cache_at(cache_path.clone(), network) {
        Ok(cache) => sns_proposals_cache_summary(cache_path, cache),
        Err(error) => invalid_sns_proposals_cache_summary(cache_path, network, &error),
    }
}

/// Build a public cache summary from one loaded complete proposal snapshot.
pub(in crate::sns::report::proposals_cache) fn sns_proposals_cache_summary(
    cache_path: PathBuf,
    cache: SnsProposalsCache,
) -> SnsCacheSummary {
    let attempt_path = attempt_path_for_cache_path(&cache_path);
    project_sns_cache_summary!(valid
        SnsCacheSummary,
        &cache_path,
        &attempt_path,
        cache
    )
}

pub(in crate::sns::report::proposals_cache) fn invalid_sns_proposals_cache_summary(
    cache_path: PathBuf,
    network: &str,
    error: &SnsHostError,
) -> SnsCacheSummary {
    let attempt_path = attempt_path_for_cache_path(&cache_path);
    project_sns_cache_summary!(invalid
        SnsCacheSummary,
        &cache_path,
        &attempt_path,
        network,
        error
    )
}
