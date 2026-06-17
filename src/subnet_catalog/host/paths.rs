//! Module: subnet_catalog::host::paths
//!
//! Responsibility: construct subnet catalog cache and lock paths under the ICP root.
//!
//! Does not own: cache read/write behavior, refresh policy, or network validation.
//!
//! Boundary: centralizes on-disk path shape for subnet catalog host operations.

use std::path::{Path, PathBuf};

/// Returns the complete catalog JSON path for a network.
#[must_use]
pub fn subnet_catalog_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("subnet-catalog")
        .join(network)
        .join("catalog.json")
}

/// Returns the refresh lock path for a network catalog.
#[must_use]
pub fn subnet_catalog_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("subnet-catalog")
        .join(network)
        .join("refresh.lock")
}
