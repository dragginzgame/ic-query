use std::path::{Path, PathBuf};

#[must_use]
pub fn subnet_catalog_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("subnet-catalog")
        .join(network)
        .join("catalog.json")
}

#[must_use]
pub fn subnet_catalog_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("subnet-catalog")
        .join(network)
        .join("refresh.lock")
}
