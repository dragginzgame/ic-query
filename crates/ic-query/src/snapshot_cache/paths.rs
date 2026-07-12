//! Module: snapshot_cache::paths
//!
//! Responsibility: build and discover snapshot cache paths.
//! Does not own: cache JSON schemas, locking, or refresh attempts.
//! Boundary: maps logical snapshot keys to `.icq` filesystem locations.

use super::SnapshotKey;
use std::{
    fs,
    path::{Path, PathBuf},
};

///
/// SnapshotJsonPaths
///
/// Filesystem paths for one complete snapshot and its refresh sidecars.
///

#[expect(
    clippy::struct_field_names,
    reason = "path suffix identifies filesystem path values in a public DTO"
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotJsonPaths {
    /// Complete snapshot JSON path.
    pub snapshot_path: PathBuf,
    /// Lock path held while replacing the snapshot.
    pub refresh_lock_path: PathBuf,
    /// Sidecar path recording the latest refresh attempt.
    pub refresh_attempt_path: PathBuf,
}

impl SnapshotJsonPaths {
    /// Build every snapshot path for one logical cache key.
    #[must_use]
    pub fn for_key(icp_root: &Path, key: &SnapshotKey) -> Self {
        let collection_dir = snapshot_collection_dir(icp_root, key);
        let file_stem = key.scope_file_stem();
        Self {
            snapshot_path: collection_dir.join(format!("{file_stem}.json")),
            refresh_lock_path: collection_dir.join(format!("{file_stem}.refresh.lock")),
            refresh_attempt_path: collection_dir.join(format!("{file_stem}.refresh-attempt.json")),
        }
    }
}

/// Return the cache directory for one domain and network.
#[must_use]
pub fn snapshot_network_dir(icp_root: &Path, domain: &str, network: &str) -> PathBuf {
    icp_root.join(".icq").join(domain).join(network)
}

/// Collect sorted complete-snapshot paths for every entity in a collection.
pub fn collect_full_collection_snapshot_paths(
    network_dir: &Path,
    collection: &str,
) -> std::io::Result<Vec<PathBuf>> {
    collect_full_collection_paths(network_dir, collection, "full.json")
}

/// Collect sorted refresh-attempt paths for every entity in a collection.
pub fn collect_full_collection_attempt_paths(
    network_dir: &Path,
    collection: &str,
) -> std::io::Result<Vec<PathBuf>> {
    collect_full_collection_paths(network_dir, collection, "full.refresh-attempt.json")
}

fn collect_full_collection_paths(
    network_dir: &Path,
    collection: &str,
    file_name: &str,
) -> std::io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(network_dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(source),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry?.path().join(collection).join(file_name);
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn snapshot_collection_dir(icp_root: &Path, key: &SnapshotKey) -> PathBuf {
    snapshot_network_dir(icp_root, key.domain(), key.network())
        .join(key.entity())
        .join(key.collection())
}
