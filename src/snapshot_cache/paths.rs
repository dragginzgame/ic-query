use super::SnapshotKey;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotJsonPaths {
    pub snapshot_path: PathBuf,
    pub refresh_lock_path: PathBuf,
    pub refresh_attempt_path: PathBuf,
}

impl SnapshotJsonPaths {
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

pub fn snapshot_network_dir(icp_root: &Path, domain: &str, network: &str) -> PathBuf {
    icp_root.join(".icq").join(domain).join(network)
}

fn snapshot_collection_dir(icp_root: &Path, key: &SnapshotKey) -> PathBuf {
    snapshot_network_dir(icp_root, key.domain(), key.network())
        .join(key.entity())
        .join(key.collection())
}
