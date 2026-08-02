//! Module: sns::report::neurons_cache::storage::summary
//!
//! Responsibility: project stored SNS neuron caches into cache summary reports.
//! Does not own: summary text rendering, cache refresh, or full report assembly.
//! Boundary: combines cache metadata with the latest refresh-attempt sidecar status.

use super::{load::load_sns_neurons_cache_at, scan::collect_sns_neurons_cache_paths};
use crate::sns::report::{
    SnsCacheSummary, SnsHostError, cache_paths::sns_attempt_path_for_cache_path,
    neurons_cache::model::SnsNeuronsCache, project_sns_cache_summary,
};
use std::path::{Path, PathBuf};

pub(in crate::sns::report::neurons_cache) fn list_sns_neurons_cache_summaries(
    cache_root: &Path,
    network: &str,
) -> Result<Vec<SnsCacheSummary>, SnsHostError> {
    collect_sns_neurons_cache_paths(cache_root, network)?
        .into_iter()
        .map(|path| Ok(load_sns_neurons_cache_summary_at(path, network)))
        .collect()
}

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_summary_at(
    cache_path: PathBuf,
    network: &str,
) -> SnsCacheSummary {
    match load_sns_neurons_cache_at(cache_path.clone(), network) {
        Ok(cache) => sns_neurons_cache_summary(cache_path, cache),
        Err(error) => invalid_sns_neurons_cache_summary(cache_path, network, &error),
    }
}

pub(in crate::sns::report::neurons_cache) fn sns_neurons_cache_summary(
    cache_path: PathBuf,
    cache: SnsNeuronsCache,
) -> SnsCacheSummary {
    let attempt_path = sns_attempt_path_for_cache_path(&cache_path);
    project_sns_cache_summary!(valid SnsCacheSummary, &cache_path, &attempt_path, cache)
}

pub(in crate::sns::report::neurons_cache) fn invalid_sns_neurons_cache_summary(
    cache_path: PathBuf,
    network: &str,
    error: &SnsHostError,
) -> SnsCacheSummary {
    let attempt_path = sns_attempt_path_for_cache_path(&cache_path);
    project_sns_cache_summary!(invalid
        SnsCacheSummary,
        &cache_path,
        &attempt_path,
        network,
        error
    )
}
