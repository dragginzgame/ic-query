use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsNeuronsCachePaths {
    pub(super) cache_path: PathBuf,
    pub(super) lock_path: PathBuf,
    pub(super) attempt_path: PathBuf,
}

pub(in crate::sns::report) fn sns_neurons_cache_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_network_cache_dir(icp_root, network)
        .join(root_canister_id)
        .join("neurons")
        .join("full.json")
}

pub(super) fn sns_network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
    icp_root.join(".icq").join("sns").join(network)
}

pub(in crate::sns::report) fn sns_neurons_refresh_lock_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_network_cache_dir(icp_root, network)
        .join(root_canister_id)
        .join("neurons")
        .join("full.refresh.lock")
}

pub(in crate::sns::report) fn sns_neurons_refresh_attempt_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_network_cache_dir(icp_root, network)
        .join(root_canister_id)
        .join("neurons")
        .join("full.refresh-attempt.json")
}

pub(super) fn sns_neurons_attempt_path_for_cache_path(cache_path: &Path) -> PathBuf {
    cache_path
        .parent()
        .expect("SNS neurons cache path always has parent")
        .join("full.refresh-attempt.json")
}
