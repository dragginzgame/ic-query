//! Module: sns::report::neurons_cache::paths
//!
//! Responsibility: own SNS neuron snapshot identity, storage admission, and cache paths.
//! Does not own: cache reads/writes, refresh policy, report construction, or rendering.
//! Boundary: maps Governance principals to generic snapshot-cache file locations.

use crate::sns::report::cache_paths::sns_snapshot_cache_paths;
use std::path::{Path, PathBuf};

#[must_use]
pub fn sns_neurons_cache_path(
    cache_root: &Path,
    network: &str,
    governance_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths(cache_root, network, governance_canister_id).snapshot_path
}

#[must_use]
pub fn sns_neurons_refresh_lock_path(
    cache_root: &Path,
    network: &str,
    governance_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths(cache_root, network, governance_canister_id).refresh_lock_path
}

#[must_use]
pub fn sns_neurons_refresh_attempt_path(
    cache_root: &Path,
    network: &str,
    governance_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths(cache_root, network, governance_canister_id).refresh_attempt_path
}
