mod cache;

pub(in crate::nns) use cache::{load_nns_leaf_json_cache, write_nns_leaf_json_refresh_cache};
use std::path::{Path, PathBuf};

///
/// NnsLeafCacheRequest
///
/// Cache identity contract shared by generic NNS leaf report requests.
///

pub(in crate::nns) trait NnsLeafCacheRequest: Clone {
    fn cache_root(&self) -> &Path;
    fn network(&self) -> &str;
}

///
/// NnsLeafRefreshRequest
///
/// Report-builder request contract for generic NNS leaf refresh operations.
///

pub(in crate::nns) trait NnsLeafRefreshRequest {
    type Cache: NnsLeafCacheRequest;

    fn cache(&self) -> &Self::Cache;
    fn now_unix_secs(&self) -> u64;
    fn lock_stale_after_seconds(&self) -> u64;
    fn dry_run(&self) -> bool;
    fn output_path(&self) -> Option<&Path>;
}

///
/// NnsLeafCachePaths
///
/// Cache and lock paths for one generic NNS leaf component snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafCachePaths {
    pub(in crate::nns) cache_path: PathBuf,
    pub(in crate::nns) lock_path: PathBuf,
}

impl NnsLeafCachePaths {
    #[must_use]
    pub(in crate::nns) fn for_component(
        cache_root: &Path,
        component_dir: &str,
        network: &str,
        cache_file: &str,
    ) -> Self {
        let cache_dir = cache_root.join(component_dir).join(network);
        Self {
            cache_path: cache_dir.join(cache_file),
            lock_path: cache_dir.join("refresh.lock"),
        }
    }
}

#[cfg(all(test, feature = "host"))]
mod tests;
