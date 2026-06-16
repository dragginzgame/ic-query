use crate::{
    nns::{NnsCommandError, command_icp_root},
    subnet_catalog::SubnetCatalogCacheRequest,
};

pub(super) fn cache_request(network: &str) -> Result<SubnetCatalogCacheRequest, NnsCommandError> {
    let icp_root = command_icp_root()?;
    Ok(SubnetCatalogCacheRequest {
        icp_root,
        network: network.to_string(),
    })
}
