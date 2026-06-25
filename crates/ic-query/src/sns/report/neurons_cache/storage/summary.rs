//! Module: sns::report::neurons_cache::storage::summary
//!
//! Responsibility: project stored SNS neuron caches into cache summary reports.
//! Does not own: summary text rendering, cache refresh, or full report assembly.
//! Boundary: combines cache metadata with the latest refresh-attempt sidecar status.

use super::{load::load_sns_neurons_cache_at, scan::collect_sns_neurons_cache_paths};
use crate::snapshot_cache::SNAPSHOT_CACHE_STATUS_OK;
use crate::sns::report::{
    SnsHostError, SnsNeuronsCacheSummary, invalid_sns_cache_summary_fields,
    neurons_cache::{
        attempt::read_sns_neurons_attempt_status, model::SnsNeuronsCache,
        paths::sns_neurons_attempt_path_for_cache_path,
    },
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
        Err(error) => invalid_sns_neurons_cache_summary(cache_path, &error),
    }
}

pub(in crate::sns::report::neurons_cache) fn sns_neurons_cache_summary(
    cache_path: PathBuf,
    cache: SnsNeuronsCache,
) -> SnsNeuronsCacheSummary {
    let attempt_path = sns_neurons_attempt_path_for_cache_path(&cache_path);
    let metadata = cache.metadata;
    SnsNeuronsCacheSummary {
        id: metadata.id,
        name: metadata.name,
        root_canister_id: metadata.root_canister_id,
        governance_canister_id: metadata.governance_canister_id,
        cache_status: SNAPSHOT_CACHE_STATUS_OK.to_string(),
        cache_error: None,
        complete: cache.completeness.is_api_exhausted(),
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_sns_neurons_attempt_status(&attempt_path),
    }
}

pub(in crate::sns::report::neurons_cache) fn invalid_sns_neurons_cache_summary(
    cache_path: PathBuf,
    error: &SnsHostError,
) -> SnsNeuronsCacheSummary {
    let attempt_path = sns_neurons_attempt_path_for_cache_path(&cache_path);
    let fields = invalid_sns_cache_summary_fields(&cache_path, &attempt_path, error);
    SnsNeuronsCacheSummary {
        id: 0,
        name: "-".to_string(),
        root_canister_id: fields.root_canister_id,
        governance_canister_id: "-".to_string(),
        cache_status: fields.cache_status,
        cache_error: fields.cache_error,
        complete: fields.complete,
        row_count: fields.row_count,
        page_count: fields.page_count,
        page_size: fields.page_size,
        fetched_at: fields.fetched_at,
        source_endpoint: fields.source_endpoint,
        cache_path: fields.cache_path,
        refresh_attempt_path: fields.refresh_attempt_path,
        latest_attempt: read_sns_neurons_attempt_status(&attempt_path),
    }
}
