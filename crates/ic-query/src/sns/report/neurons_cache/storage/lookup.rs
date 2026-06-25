//! Module: sns::report::neurons_cache::storage::lookup
//!
//! Responsibility: resolve SNS neuron cache input to a complete cached snapshot.
//! Does not own: CLI argument parsing, refresh collection, cache rendering, or live fetches.
//! Boundary: supports numeric SNS ids and root canister principals over local cache files.

use super::{
    errors::{invalid_lookup_error, missing_id_error},
    load::load_sns_neurons_cache_at,
    scan::{collect_sns_neurons_cache_paths, read_sns_neurons_cache_header},
};
use crate::sns::report::{
    SnsHostError, enforce_mainnet_network,
    neurons_cache::{
        model::SnsNeuronsCache,
        paths::{sns_network_cache_dir, sns_neurons_cache_path},
    },
    parse_sns_root_canister_input,
};
use std::path::{Path, PathBuf};

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_for_input(
    icp_root: &Path,
    network: &str,
    input: &str,
) -> Result<(PathBuf, SnsNeuronsCache), SnsHostError> {
    enforce_mainnet_network(network)?;
    if let Ok(id) = input.parse::<usize>() {
        return find_sns_neurons_cache_by_id(icp_root, network, id)?
            .ok_or_else(|| missing_id_error(id, sns_network_cache_dir(icp_root, network)));
    }

    let root_canister_id =
        parse_sns_root_canister_input(input).map_err(|_| invalid_lookup_error(input))?;
    let path = sns_neurons_cache_path(icp_root, network, &root_canister_id);
    let cache = load_sns_neurons_cache_at(path.clone(), network)?;
    Ok((path, cache))
}

pub(in crate::sns::report::neurons_cache) fn find_sns_neurons_cache_by_id(
    icp_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<(PathBuf, SnsNeuronsCache)>, SnsHostError> {
    for path in collect_sns_neurons_cache_paths(icp_root, network)? {
        let header = read_sns_neurons_cache_header(&path, network)?;
        if header.metadata.id == id {
            let cache = load_sns_neurons_cache_at(path.clone(), network)?;
            return Ok(Some((path, cache)));
        }
    }
    Ok(None)
}
