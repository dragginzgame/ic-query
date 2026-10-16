use crate::{nns::NnsInventoryCacheRequest, subnet_catalog::SubnetCatalogCacheRequest};

pub(in crate::nns::topology::report) fn subnet_catalog_cache_request(
    request: &super::model::NnsTopologyReadRequest,
) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest::new(request.cache_root.as_path(), request.network.as_str())
}

pub(in crate::nns::topology::report) fn inventory_cache_request(
    request: &super::model::NnsTopologyReadRequest,
) -> NnsInventoryCacheRequest {
    NnsInventoryCacheRequest::new(request.cache_root.as_path(), request.network.as_str())
}
