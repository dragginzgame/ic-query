use crate::nns::{leaf::NnsLeafCacheRequest, node::report::NnsNodeCacheRequest};
use std::path::Path;

pub(super) fn cache_request(icp_root: &Path, network: &str) -> NnsNodeCacheRequest {
    NnsNodeCacheRequest::from_root_network(icp_root, network)
}
