mod model;

pub use model::NnsTopologyRefreshRequest;

#[cfg(feature = "nns-host")]
use crate::{
    nns::{NnsInventoryListRequest, NnsInventoryRefreshRequest},
    subnet_catalog::{
        CatalogSourceSelection, DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogCacheRequest,
        SubnetCatalogListRequest, SubnetCatalogRefreshRequest,
    },
};

#[cfg(feature = "nns-host")]
pub(super) fn subnet_catalog_cache_request(
    request: &NnsInventoryListRequest,
) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest::new(
        request.cache.cache_root.as_path(),
        request.cache.network.as_str(),
    )
}

#[cfg(feature = "nns-host")]
pub(super) fn subnet_catalog_list_request(
    request: &NnsInventoryListRequest,
) -> SubnetCatalogListRequest {
    let mut configured_request = SubnetCatalogListRequest::new(
        subnet_catalog_cache_request(request),
        request.source_endpoint.as_str(),
        request.now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    configured_request.range_limit = 1;
    configured_request
}

#[cfg(feature = "nns-host")]
pub(super) fn subnet_catalog_refresh_request(
    request: &NnsTopologyRefreshRequest,
) -> SubnetCatalogRefreshRequest {
    let mut configured_request = SubnetCatalogRefreshRequest::new(
        subnet_catalog_cache_request(&request.read),
        CatalogSourceSelection::uncertified_query(request.read.source_endpoint.as_str()),
        request.read.now_unix_secs,
    );
    configured_request.dry_run = request.dry_run;
    configured_request
}

#[cfg(feature = "nns-host")]
pub(super) fn inventory_refresh_request(
    request: &NnsTopologyRefreshRequest,
) -> NnsInventoryRefreshRequest {
    let mut configured_request = NnsInventoryRefreshRequest::new(
        request.read.cache.clone(),
        request.read.source_endpoint.as_str(),
        request.read.now_unix_secs,
    );
    configured_request.dry_run = request.dry_run;
    configured_request
}
