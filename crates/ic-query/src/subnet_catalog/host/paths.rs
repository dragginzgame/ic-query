//! Module: subnet_catalog::host::paths
//!
//! Responsibility: construct subnet catalog cache and lock paths under the cache root.
//!
//! Does not own: cache read/write behavior, refresh policy, or network validation.
//!
//! Boundary: centralizes on-disk path shape for subnet catalog host operations.

use std::path::{Path, PathBuf};

/// Returns the complete catalog JSON path for a network.
#[must_use]
pub fn subnet_catalog_path(cache_root: &Path, network: &str) -> PathBuf {
    cache_root
        .join("nns")
        .join(network)
        .join("subnet-catalog")
        .join("catalog.json")
}

/// Returns the refresh lock path for a network catalog.
#[must_use]
pub fn subnet_catalog_refresh_lock_path(cache_root: &Path, network: &str) -> PathBuf {
    cache_root
        .join("nns")
        .join(network)
        .join("subnet-catalog")
        .join("refresh.lock")
}
