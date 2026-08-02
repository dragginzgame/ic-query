//! Module: sns::report::neurons_cache::storage::lookup
//!
//! Responsibility: resolve SNS neuron cache input to a complete cached snapshot.
//! Does not own: CLI argument parsing, refresh collection, cache rendering, or live fetches.
//! Boundary: supports numeric SNS ids and root canister principals over local cache files.

use super::errors::{invalid_lookup_error, missing_id_error};
use crate::sns::report::{
    SnsHostError,
    cache_paths::sns_snapshot_network_cache_dir,
    cache_storage::{
        collect_sns_cache_paths, find_unique_sns_cache_path_by_id, load_sns_cache_at,
        read_sns_cache_header,
    },
    enforce_mainnet_network,
    neurons_cache::{
        model::SnsNeuronsCache,
        paths::{SnsNeuronsCacheCollection, sns_neurons_cache_path},
    },
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
        return find_sns_neurons_cache_by_id(cache_root, network, id)?.ok_or_else(|| {
            missing_id_error(id, sns_snapshot_network_cache_dir(cache_root, network))
        });
    }

    let root_canister_id =
        parse_sns_root_canister_input(input).map_err(|_| invalid_lookup_error(input))?;
    let path = sns_neurons_cache_path(cache_root, network, &root_canister_id);
    let cache = load_sns_cache_at::<SnsNeuronsCacheCollection>(path.clone(), network)?;
    Ok((path, cache))
}

pub(in crate::sns::report::neurons_cache) fn find_sns_neurons_cache_by_id(
    cache_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<(PathBuf, SnsNeuronsCache)>, SnsHostError> {
    let path = find_unique_sns_cache_path_by_id(
        collect_sns_cache_paths::<SnsNeuronsCacheCollection>(cache_root, network)?,
        id,
        |path| {
            read_sns_cache_header::<SnsNeuronsCacheCollection>(path, network)
                .map(|header| header.metadata.id)
        },
    )?;
    path.map(|path| {
        load_sns_cache_at::<SnsNeuronsCacheCollection>(path.clone(), network)
            .map(|cache| (path, cache))
    })
    .transpose()
}
