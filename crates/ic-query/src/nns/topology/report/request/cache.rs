use crate::nns::NnsInventoryListRequest;
use crate::subnet_catalog::SubnetCatalogCacheRequest;

pub(in crate::nns::topology::report) fn subnet_catalog_cache_request(
    request: &NnsInventoryListRequest,
) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest::new(
        request.cache.cache_root.as_path(),
        request.cache.network.as_str(),
    )
}
