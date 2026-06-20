//! Module: sns::report::neurons_cache::storage::summary
//!
//! Responsibility: project stored SNS neuron caches into cache summary reports.
//! Does not own: summary text rendering, cache refresh, or full report assembly.
//! Boundary: combines cache metadata with the latest refresh-attempt sidecar status.

use super::{load::load_sns_neurons_cache_at, scan::collect_sns_neurons_cache_paths};
use crate::snapshot_cache::{SNAPSHOT_CACHE_STATUS_INVALID, SNAPSHOT_CACHE_STATUS_OK};
use crate::sns::report::{
    SnsHostError, SnsNeuronsCacheSummary,
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
        .map(|path| {
            Ok(match load_sns_neurons_cache_at(path.clone(), network) {
                Ok(cache) => sns_neurons_cache_summary(path, cache),
                Err(error) => invalid_sns_neurons_cache_summary(path, error),
            })
        })
        .collect()
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
    error: SnsHostError,
) -> SnsNeuronsCacheSummary {
    let attempt_path = sns_neurons_attempt_path_for_cache_path(&cache_path);
    SnsNeuronsCacheSummary {
        id: 0,
        name: "-".to_string(),
        root_canister_id: root_from_cache_path(&cache_path),
        governance_canister_id: "-".to_string(),
        cache_status: SNAPSHOT_CACHE_STATUS_INVALID.to_string(),
        cache_error: Some(error.to_string()),
        complete: false,
        row_count: 0,
        page_count: 0,
        page_size: 0,
        fetched_at: "-".to_string(),
        source_endpoint: "-".to_string(),
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_sns_neurons_attempt_status(&attempt_path),
    }
}

fn root_from_cache_path(cache_path: &Path) -> String {
    cache_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map_or_else(
            || "-".to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
}
