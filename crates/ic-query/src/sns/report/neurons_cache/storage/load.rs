//! Module: sns::report::neurons_cache::storage::load
//!
//! Responsibility: load one complete SNS neuron cache snapshot from disk.
//! Does not own: lookup resolution, cache path construction, refresh, or rendering.
//! Boundary: validates schema, network, and completeness before returning a cache model.

use crate::snapshot_cache::validate_snapshot_completeness;
use crate::sns::report::{
    SnsHostError,
    cache_paths::sns_snapshot_key_for_cache_path,
    cache_storage::{load_sns_complete_cache, validate_sns_cache_metadata},
    neurons_cache::{model::SnsNeuronsCache, paths::SnsNeuronsCacheCollection},
    source::validate_sns_neuron_rows,
};
use std::path::{Path, PathBuf};

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_at(
    path: PathBuf,
    network: &str,
) -> Result<SnsNeuronsCache, SnsHostError> {
    let key = sns_snapshot_key_for_cache_path::<SnsNeuronsCacheCollection>(network, &path);
    let cache = load_sns_complete_cache::<SnsNeuronsCache, SnsNeuronsCacheCollection>(
        path.clone(),
        network,
        &key,
    )?;
    validate_sns_neurons_cache(&path, &cache)?;
    Ok(cache)
}

fn validate_sns_neurons_cache(path: &Path, cache: &SnsNeuronsCache) -> Result<(), SnsHostError> {
    let invalid = |reason| SnsHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    validate_snapshot_completeness(&cache.completeness, cache.data.neurons.len())
        .map_err(invalid)?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "SNS Governance neuron pagination cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    validate_sns_cache_metadata(path, &cache.metadata, &cache.entity)?;
    validate_sns_neuron_rows(&cache.data.neurons).map_err(invalid)?;
    Ok(())
}
