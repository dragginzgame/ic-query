//! Module: sns::report::neurons_cache::paths
//!
//! Responsibility: construct SNS neuron snapshot cache paths.
//! Does not own: cache reads/writes, refresh policy, report construction, or rendering.
//! Boundary: maps SNS root principals to generic snapshot-cache file locations.

use crate::snapshot_cache::SnapshotKey;
use crate::sns::report::cache_paths::{
    SnsCacheCollection, SnsSnapshotCachePaths, sns_attempt_path_for_cache_path,
    sns_snapshot_key_for_cache_path, sns_snapshot_network_cache_dir,
};
use std::path::{Path, PathBuf};

pub(super) type SnsNeuronsCachePaths = SnsSnapshotCachePaths<SnsNeuronsCacheCollection>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SnsNeuronsCacheCollection {}

impl SnsCacheCollection for SnsNeuronsCacheCollection {
    const COLLECTION: &'static str = "neurons";
}

#[must_use]
pub fn sns_neurons_cache_path(icp_root: &Path, network: &str, root_canister_id: &str) -> PathBuf {
    SnsNeuronsCachePaths::for_root(icp_root, network, root_canister_id).cache_path
}

pub(super) fn sns_network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
    sns_snapshot_network_cache_dir(icp_root, network)
}

#[must_use]
pub fn sns_neurons_refresh_lock_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    SnsNeuronsCachePaths::for_root(icp_root, network, root_canister_id).lock_path
}

#[must_use]
pub fn sns_neurons_refresh_attempt_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    SnsNeuronsCachePaths::for_root(icp_root, network, root_canister_id).attempt_path
}

pub(super) fn sns_neurons_attempt_path_for_cache_path(cache_path: &Path) -> PathBuf {
    sns_attempt_path_for_cache_path(cache_path)
}

pub(super) fn sns_neurons_cache_key_for_cache_path(
    network: &str,
    cache_path: &Path,
) -> SnapshotKey {
    sns_snapshot_key_for_cache_path::<SnsNeuronsCacheCollection>(network, cache_path)
}
