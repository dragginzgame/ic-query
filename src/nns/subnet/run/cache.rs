use crate::subnet_catalog::SubnetCatalogCacheRequest;
use std::path::{Path, PathBuf};

pub(super) fn cache_request(icp_root: &Path, network: &str) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest {
        icp_root: PathBuf::from(icp_root),
        network: network.to_string(),
    }
}
