//! Module: sns::report::neurons_cache::storage::summary
//!
//! Responsibility: project stored SNS neuron caches into cache summary reports.
//! Does not own: summary text rendering, cache refresh, or full report assembly.
//! Boundary: combines cache metadata with the latest refresh-attempt sidecar status.

use super::{load::load_sns_neurons_cache_at, scan::collect_sns_neurons_cache_paths};
use crate::sns::report::{
    SnsHostError, SnsNeuronsCacheSummary,
    neurons_cache::{model::SnsNeuronsCache, paths::sns_neurons_attempt_path_for_cache_path},
    project_sns_cache_summary,
};
use std::path::{Path, PathBuf};

pub(in crate::sns::report::neurons_cache) fn list_sns_neurons_cache_summaries(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<SnsNeuronsCacheSummary>, SnsHostError> {
    collect_sns_neurons_cache_paths(icp_root, network)?
        .into_iter()
        .map(|path| Ok(load_sns_neurons_cache_summary_at(path, network)))
        .collect()
}

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_summary_at(
    cache_path: PathBuf,
    network: &str,
) -> SnsNeuronsCacheSummary {
    match load_sns_neurons_cache_at(cache_path.clone(), network) {
        Ok(cache) => sns_neurons_cache_summary(cache_path, cache),
        Err(error) => invalid_sns_neurons_cache_summary(cache_path, network, &error),
    }
}

pub(in crate::sns::report::neurons_cache) fn sns_neurons_cache_summary(
    cache_path: PathBuf,
    cache: SnsNeuronsCache,
) -> SnsNeuronsCacheSummary {
    let attempt_path = sns_neurons_attempt_path_for_cache_path(&cache_path);
    project_sns_cache_summary!(valid SnsNeuronsCacheSummary, &cache_path, &attempt_path, cache)
}

pub(in crate::sns::report::neurons_cache) fn invalid_sns_neurons_cache_summary(
    cache_path: PathBuf,
    network: &str,
    error: &SnsHostError,
) -> SnsNeuronsCacheSummary {
    let attempt_path = sns_neurons_attempt_path_for_cache_path(&cache_path);
    project_sns_cache_summary!(invalid
        SnsNeuronsCacheSummary,
        &cache_path,
        &attempt_path,
        network,
        error
    )
}
