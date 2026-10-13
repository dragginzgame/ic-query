//! Module: sns::report::proposals_cache::paths
//!
//! Responsibility: proposal snapshot cache path construction.
//! Does not own: cache loading, refresh locking, or status rendering.
//! Boundary: maps SNS root principals to proposal snapshot file paths.

use crate::sns::report::cache_paths::{SnsCacheCollection, sns_snapshot_cache_paths};
use std::path::{Path, PathBuf};

///
/// SnsProposalsCacheCollection
///
/// Collection marker used to derive complete SNS proposal snapshot paths.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) enum SnsProposalsCacheCollection {}

impl SnsCacheCollection for SnsProposalsCacheCollection {
    const COLLECTION: &'static str = "proposals";
}

#[must_use]
pub fn sns_proposals_cache_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsProposalsCacheCollection>(cache_root, network, root_canister_id)
        .snapshot_path
}

#[must_use]
pub fn sns_proposals_refresh_lock_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsProposalsCacheCollection>(cache_root, network, root_canister_id)
        .refresh_lock_path
}

#[must_use]
pub fn sns_proposals_refresh_attempt_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsProposalsCacheCollection>(cache_root, network, root_canister_id)
        .refresh_attempt_path
}
