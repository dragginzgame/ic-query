//! Module: sns::report::neurons_cache::storage::load
//!
//! Responsibility: load one complete SNS neuron cache snapshot from disk.
//! Does not own: lookup resolution, cache path construction, refresh, or rendering.
//! Boundary: validates schema, network, and completeness before returning a cache model.

use crate::snapshot_cache::validate_snapshot_completeness;
use crate::sns::report::{
    SnsHostError,
    cache_storage::{SnsCacheLoadErrors, load_sns_complete_cache, validate_sns_cache_metadata},
    neurons_cache::{
        SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCache,
        paths::sns_neurons_cache_key_for_cache_path,
    },
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_at(
    path: PathBuf,
    network: &str,
) -> Result<SnsNeuronsCache, SnsHostError> {
    let key = sns_neurons_cache_key_for_cache_path(network, &path);
    let errors = SnsCacheLoadErrors::neurons();
    let cache = load_sns_complete_cache(
        path.clone(),
        network,
        SNS_NEURONS_CACHE_SCHEMA_VERSION,
        &key,
        errors,
        |completeness| errors.incomplete_cache_error(completeness),
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
    validate_sns_cache_metadata(path, &cache.metadata, &cache.entity)?;
    let mut neuron_ids = HashSet::new();
    for neuron in &cache.data.neurons {
        if neuron.neuron_id.is_empty()
            || !neuron.neuron_id.len().is_multiple_of(2)
            || !neuron
                .neuron_id
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(invalid("cache contains an invalid neuron id".to_string()));
        }
        if !neuron_ids.insert(&neuron.neuron_id) {
            return Err(invalid(format!("duplicate neuron id {}", neuron.neuron_id)));
        }
    }
    Ok(())
}
