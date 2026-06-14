use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsNeuronsCachePaths {
    pub(super) cache_path: PathBuf,
    pub(super) lock_path: PathBuf,
    pub(super) attempt_path: PathBuf,
}

impl SnsNeuronsCachePaths {
    pub(super) fn for_root(icp_root: &Path, network: &str, root_canister_id: &str) -> Self {
        let cache_dir = sns_neurons_cache_dir(icp_root, network, root_canister_id);
        Self {
            cache_path: cache_dir.join("full.json"),
            lock_path: cache_dir.join("full.refresh.lock"),
            attempt_path: cache_dir.join("full.refresh-attempt.json"),
        }
    }
}

pub(in crate::sns::report) fn sns_neurons_cache_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    SnsNeuronsCachePaths::for_root(icp_root, network, root_canister_id).cache_path
}

pub(super) fn sns_network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
    icp_root.join(".icq").join("sns").join(network)
}

#[cfg(test)]
pub(in crate::sns::report) fn sns_neurons_refresh_lock_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    SnsNeuronsCachePaths::for_root(icp_root, network, root_canister_id).lock_path
}

#[cfg(test)]
pub(in crate::sns::report) fn sns_neurons_refresh_attempt_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    SnsNeuronsCachePaths::for_root(icp_root, network, root_canister_id).attempt_path
}

pub(super) fn sns_neurons_attempt_path_for_cache_path(cache_path: &Path) -> PathBuf {
    cache_path.with_file_name("full.refresh-attempt.json")
}

fn sns_neurons_cache_dir(icp_root: &Path, network: &str, root_canister_id: &str) -> PathBuf {
    sns_network_cache_dir(icp_root, network)
        .join(root_canister_id)
        .join("neurons")
}
