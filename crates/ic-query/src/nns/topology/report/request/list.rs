use super::cache::subnet_catalog_cache_request;
use crate::nns::NnsInventoryListRequest;
use crate::{
    nns::node::NnsNodeListRequest,
    subnet_catalog::{DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogListRequest},
};

pub(in crate::nns::topology::report) fn subnet_catalog_list_request(
    request: &NnsInventoryListRequest,
) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest::new(
        subnet_catalog_cache_request(request),
        request.source_endpoint.as_str(),
        request.now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_range_limit(1)
}

pub(in crate::nns::topology::report) fn node_list_request(
    request: &NnsInventoryListRequest,
) -> NnsNodeListRequest {
    NnsNodeListRequest::new(
        request.cache.clone(),
        request.source_endpoint.as_str(),
        request.now_unix_secs,
    )
}
