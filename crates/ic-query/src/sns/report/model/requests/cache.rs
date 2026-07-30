//! Module: sns::report::model::requests::cache
//!
//! Responsibility: shared requests for inspecting SNS complete-snapshot caches.
//! Does not own: collection-specific cache paths, report construction, or rendering.
//! Boundary: carries cache identity shared by neuron and proposal cache commands.

use std::path::{Path, PathBuf};

///
/// SnsCacheListRequest
///
/// Request accepted by an SNS complete-snapshot cache list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsCacheListRequest {
    pub network: String,
    pub cache_root: PathBuf,
}

impl SnsCacheListRequest {
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            network: network.into(),
            cache_root: cache_root.into(),
        }
    }

    #[must_use]
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }
}

///
/// SnsCacheStatusRequest
///
/// Request accepted by an SNS complete-snapshot cache status report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsCacheStatusRequest {
    pub network: String,
    pub cache_root: PathBuf,
    pub input: String,
}

impl SnsCacheStatusRequest {
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            cache_root: cache_root.into(),
            input: input.into(),
        }
    }

    #[must_use]
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }
}
