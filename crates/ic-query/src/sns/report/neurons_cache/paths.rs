//! Module: sns::report::neurons_cache::paths
//!
//! Responsibility: construct SNS neuron snapshot cache paths.
//! Does not own: cache reads/writes, refresh policy, report construction, or rendering.
//! Boundary: maps SNS root principals to generic snapshot-cache file locations.

use crate::sns::report::cache_paths::sns_snapshot_cache_paths;
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
