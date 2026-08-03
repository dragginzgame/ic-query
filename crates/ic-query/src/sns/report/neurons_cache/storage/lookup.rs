//! Module: sns::report::neurons_cache::storage::lookup
//!
//! Responsibility: resolve SNS neuron cache input to a complete cached snapshot.
//! Does not own: CLI argument parsing, refresh collection, cache rendering, or live fetches.
//! Boundary: supports numeric SNS ids and root canister principals over local cache files.

use crate::sns::report::{
    SnsHostError,
    cache_paths::sns_snapshot_network_cache_dir,
    cache_storage::{load_sns_cache_by_id, load_sns_cache_for_root},
    enforce_mainnet_network,
    neurons_cache::{model::SnsNeuronsCache, paths::SnsNeuronsCacheCollection},
    parse_sns_root_canister_input,
};
use std::path::{Path, PathBuf};

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_for_input(
    cache_root: &Path,
    network: &str,
    input: &str,
) -> Result<(PathBuf, SnsNeuronsCache), SnsHostError> {
    enforce_mainnet_network(network)?;
    if let Ok(id) = input.parse::<usize>() {
        return load_sns_cache_by_id::<SnsNeuronsCacheCollection>(cache_root, network, id)?
            .ok_or_else(|| SnsHostError::MissingNeuronsCacheForId {
                id,
                root: sns_snapshot_network_cache_dir(cache_root, network),
            });
    }

    let root_canister_id = parse_sns_root_canister_input(input)?;
    load_sns_cache_for_root::<SnsNeuronsCacheCollection>(cache_root, network, &root_canister_id)
}
