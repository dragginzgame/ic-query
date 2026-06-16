use crate::nns::{
    NnsCommandError, command_icp_root, leaf::NnsLeafCacheRequest, node::report::NnsNodeCacheRequest,
};

pub(super) fn cache_request(network: &str) -> Result<NnsNodeCacheRequest, NnsCommandError> {
    let icp_root = command_icp_root()?;
    Ok(NnsNodeCacheRequest::from_root_network(&icp_root, network))
}
