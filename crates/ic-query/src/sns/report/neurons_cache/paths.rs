//! Module: sns::report::neurons_cache::paths
//!
//! Responsibility: own SNS neuron snapshot identity, storage admission, and cache paths.
//! Does not own: cache reads/writes, refresh policy, report construction, or rendering.
//! Boundary: maps SNS root principals to generic snapshot-cache file locations.

use crate::sns::report::{
    SnsHostError,
    cache_paths::sns_snapshot_cache_paths,
    cache_storage::SnsCacheStorageFamily,
    neurons_cache::{SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCacheRows},
    source::validate_sns_neuron_rows,
};
use std::path::{Path, PathBuf};

///
/// SnsNeuronsCacheCollection
///
/// Collection marker used to derive complete SNS neuron snapshot paths.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SnsNeuronsCacheCollection {}

#[must_use]
pub fn sns_neurons_cache_path(cache_root: &Path, network: &str, root_canister_id: &str) -> PathBuf {
    sns_snapshot_cache_paths::<SnsNeuronsCacheCollection>(cache_root, network, root_canister_id)
        .snapshot_path
}

#[must_use]
pub fn sns_neurons_refresh_lock_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsNeuronsCacheCollection>(cache_root, network, root_canister_id)
        .refresh_lock_path
}

#[must_use]
pub fn sns_neurons_refresh_attempt_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsNeuronsCacheCollection>(cache_root, network, root_canister_id)
        .refresh_attempt_path
}

impl SnsCacheStorageFamily for SnsNeuronsCacheCollection {
    const COLLECTION: &'static str = "neurons";

    type Data = SnsNeuronsCacheRows;

    const CACHE_SCHEMA_VERSION: u32 = SNS_NEURONS_CACHE_SCHEMA_VERSION;

    const CACHE_ITEM_NAME: &'static str = "neuron";

    fn missing_cache_error(path: PathBuf) -> SnsHostError {
        SnsHostError::MissingNeuronsCache { path }
    }

    fn row_count(data: &Self::Data) -> usize {
        data.neurons.len()
    }

    fn validate_rows(data: &Self::Data) -> Result<(), String> {
        validate_sns_neuron_rows(&data.neurons)
    }
}
