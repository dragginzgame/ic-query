use super::{
    cache::{inventory_cache_request, subnet_catalog_cache_request},
    model::TopologyRequestParts,
};
use crate::{
    nns::{NnsInventoryListRequest, node::NnsNodeListRequest},
    subnet_catalog::{DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogListRequest},
};

pub(in crate::nns::topology::report) fn subnet_catalog_list_request(
    request: &impl TopologyRequestParts,
) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest::new(
        subnet_catalog_cache_request(request),
        request.source_endpoint(),
        request.now_unix_secs(),
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_range_limit(1)
}

pub(in crate::nns::topology::report) fn node_list_request(
    request: &impl TopologyRequestParts,
) -> NnsNodeListRequest {
    NnsNodeListRequest::new(
        inventory_cache_request(request),
        request.source_endpoint(),
        request.now_unix_secs(),
    )
}

pub(in crate::nns::topology::report) fn inventory_list_request(
    request: &impl TopologyRequestParts,
) -> NnsInventoryListRequest {
    NnsInventoryListRequest::new(
        inventory_cache_request(request),
        request.source_endpoint(),
        request.now_unix_secs(),
    )
}
