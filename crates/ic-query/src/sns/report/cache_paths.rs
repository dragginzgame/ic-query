//! Module: sns::report::cache_paths
//!
//! Responsibility: shared SNS snapshot cache path construction.
//! Does not own: cache reads, writes, refresh locking, or report construction.
//! Boundary: maps Governance principals and collection names to snapshot-cache paths.

use crate::snapshot_cache::{SnapshotJsonPaths, SnapshotKey, snapshot_network_dir};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Build complete snapshot, lock, and refresh-attempt paths for one Governance target.
pub(in crate::sns::report) fn sns_snapshot_cache_paths(
    cache_root: &Path,
    network: &str,
    governance_canister_id: &str,
) -> SnapshotJsonPaths {
    let snapshot_key = SnapshotKey::full("sns-neuron", network, governance_canister_id, "neurons");
    SnapshotJsonPaths::for_key(cache_root, &snapshot_key)
}

/// Return the network-level SNS snapshot cache directory.
pub(in crate::sns::report) fn sns_snapshot_network_cache_dir(
    cache_root: &Path,
    network: &str,
) -> PathBuf {
    snapshot_network_dir(cache_root, "sns-neuron", network)
}

/// Return the logical key expected for a complete SNS cache path.
pub(in crate::sns::report) fn sns_snapshot_key_for_cache_path(
    network: &str,
    cache_path: &Path,
) -> SnapshotKey {
    let entity = sns_governance_from_cache_path(cache_path)
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    SnapshotKey::full("sns-neuron", network, entity, "neurons")
}

/// Borrow the Governance directory identity shared by discovered snapshot readers.
pub(in crate::sns::report) fn sns_governance_from_cache_path(cache_path: &Path) -> Option<&OsStr> {
    cache_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
}
