//! Module: sns::report::cache_paths
//!
//! Responsibility: shared SNS snapshot cache path construction.
//! Does not own: cache reads, writes, refresh locking, or report construction.
//! Boundary: maps SNS root principals and collection names to snapshot-cache paths.

use crate::snapshot_cache::{SnapshotJsonPaths, SnapshotKey, snapshot_network_dir};
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

///
/// SnsCacheCollection
///
/// Collection marker implemented by SNS cache families that share snapshot
/// path construction.
///

pub(in crate::sns::report) trait SnsCacheCollection {
    const COLLECTION: &'static str;
}

///
/// SnsSnapshotCachePaths
///
/// Filesystem paths for one complete SNS snapshot cache and refresh state.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct SnsSnapshotCachePaths<Collection> {
    pub(in crate::sns::report) cache_path: PathBuf,
    pub(in crate::sns::report) lock_path: PathBuf,
    pub(in crate::sns::report) attempt_path: PathBuf,
    collection: PhantomData<Collection>,
}

impl<Collection> SnsSnapshotCachePaths<Collection>
where
    Collection: SnsCacheCollection,
{
    /// Build complete snapshot, lock, and refresh-attempt paths for one SNS root.
    pub(in crate::sns::report) fn for_root(
        icp_root: &Path,
        network: &str,
        root_canister_id: &str,
    ) -> Self {
        let snapshot_key =
            SnapshotKey::full("sns", network, root_canister_id, Collection::COLLECTION);
        let snapshot_paths = SnapshotJsonPaths::for_key(icp_root, &snapshot_key);
        Self {
            cache_path: snapshot_paths.snapshot_path,
            lock_path: snapshot_paths.refresh_lock_path,
            attempt_path: snapshot_paths.refresh_attempt_path,
            collection: PhantomData,
        }
    }
}

/// Return the network-level SNS snapshot cache directory.
pub(in crate::sns::report) fn sns_snapshot_network_cache_dir(
    icp_root: &Path,
    network: &str,
) -> PathBuf {
    snapshot_network_dir(icp_root, "sns", network)
}

/// Return the refresh-attempt path paired with a complete cache file.
pub(in crate::sns::report) fn sns_attempt_path_for_cache_path(cache_path: &Path) -> PathBuf {
    cache_path.with_file_name("full.refresh-attempt.json")
}

/// Return the logical key expected for a complete SNS cache path.
pub(in crate::sns::report) fn sns_snapshot_key_for_cache_path<Collection>(
    network: &str,
    cache_path: &Path,
) -> SnapshotKey
where
    Collection: SnsCacheCollection,
{
    let entity = cache_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    SnapshotKey::full("sns", network, entity, Collection::COLLECTION)
}
