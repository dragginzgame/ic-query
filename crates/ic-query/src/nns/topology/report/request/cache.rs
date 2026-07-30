use super::model::TopologyRequestParts;
use crate::{nns::NnsInventoryCacheRequest, subnet_catalog::SubnetCatalogCacheRequest};

pub(in crate::nns::topology::report) fn subnet_catalog_cache_request(
    request: &impl TopologyRequestParts,
) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest::new(request.cache_root(), request.network())
}

pub(in crate::nns::topology::report) fn inventory_cache_request(
    request: &impl TopologyRequestParts,
) -> NnsInventoryCacheRequest {
    NnsInventoryCacheRequest::new(request.cache_root(), request.network())
}
