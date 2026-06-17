//! Module: sns::report::proposals_cache::paths
//!
//! Responsibility: proposal snapshot cache path construction.
//! Does not own: cache loading, refresh locking, or status rendering.
//! Boundary: maps SNS root principals to proposal snapshot file paths.

use crate::snapshot_cache::{SnapshotJsonPaths, SnapshotKey, snapshot_network_dir};
use std::path::{Path, PathBuf};

///
/// SnsProposalsCachePaths
///
/// Filesystem paths for one complete SNS proposal snapshot and refresh state.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsProposalsCachePaths {
    pub(super) cache_path: PathBuf,
    pub(super) lock_path: PathBuf,
    pub(super) attempt_path: PathBuf,
}

impl SnsProposalsCachePaths {
    /// Build proposal cache paths for one SNS root principal.
    pub(super) fn for_root(icp_root: &Path, network: &str, root_canister_id: &str) -> Self {
        let snapshot_key = SnapshotKey::full("sns", network, root_canister_id, "proposals");
        let snapshot_paths = SnapshotJsonPaths::for_key(icp_root, &snapshot_key);
        Self {
            cache_path: snapshot_paths.snapshot_path,
            lock_path: snapshot_paths.refresh_lock_path,
            attempt_path: snapshot_paths.refresh_attempt_path,
        }
    }
}

/// Return the network-level SNS cache directory.
pub(super) fn sns_network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
    snapshot_network_dir(icp_root, "sns", network)
}

/// Return the refresh-attempt path paired with a proposal cache path.
pub(super) fn attempt_path_for_cache_path(cache_path: &Path) -> PathBuf {
    cache_path.with_file_name("full.refresh-attempt.json")
}
