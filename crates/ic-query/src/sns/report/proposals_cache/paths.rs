//! Module: sns::report::proposals_cache::paths
//!
//! Responsibility: proposal snapshot cache path construction.
//! Does not own: cache loading, refresh locking, or status rendering.
//! Boundary: maps SNS root principals to proposal snapshot file paths.

use crate::snapshot_cache::SnapshotKey;
use crate::sns::report::cache_paths::{
    SnsCacheCollection, SnsSnapshotCachePaths, sns_attempt_path_for_cache_path,
    sns_snapshot_key_for_cache_path, sns_snapshot_network_cache_dir,
};
use std::path::{Path, PathBuf};

pub(super) type SnsProposalsCachePaths = SnsSnapshotCachePaths<SnsProposalsCacheCollection>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SnsProposalsCacheCollection {}

impl SnsCacheCollection for SnsProposalsCacheCollection {
    const COLLECTION: &'static str = "proposals";
}

/// Return the network-level SNS cache directory.
pub(super) fn sns_network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
    sns_snapshot_network_cache_dir(icp_root, network)
}

/// Return the refresh-attempt path paired with a proposal cache path.
pub(super) fn attempt_path_for_cache_path(cache_path: &Path) -> PathBuf {
    sns_attempt_path_for_cache_path(cache_path)
}

pub(super) fn sns_proposals_cache_key_for_cache_path(
    network: &str,
    cache_path: &Path,
) -> SnapshotKey {
    sns_snapshot_key_for_cache_path::<SnsProposalsCacheCollection>(network, cache_path)
}
