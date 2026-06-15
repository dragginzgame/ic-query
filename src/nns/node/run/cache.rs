use crate::nns::node::report::NnsNodeCacheRequest;
use std::path::{Path, PathBuf};

pub(super) fn cache_request(icp_root: &Path, network: &str) -> NnsNodeCacheRequest {
    NnsNodeCacheRequest {
        icp_root: PathBuf::from(icp_root),
        network: network.to_string(),
    }
}
