use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafCachePaths {
    pub(in crate::nns) cache_path: PathBuf,
    pub(in crate::nns) lock_path: PathBuf,
}

impl NnsLeafCachePaths {
    #[must_use]
    pub(in crate::nns) fn for_component(
        icp_root: &Path,
        component_dir: &str,
        network: &str,
        cache_file: &str,
    ) -> Self {
        let cache_dir = nns_leaf_cache_dir(icp_root, component_dir, network);
        Self {
            cache_path: cache_dir.join(cache_file),
            lock_path: cache_dir.join("refresh.lock"),
        }
    }
}

fn nns_leaf_cache_dir(icp_root: &Path, component_dir: &str, network: &str) -> PathBuf {
    icp_root.join(".icq").join(component_dir).join(network)
}
