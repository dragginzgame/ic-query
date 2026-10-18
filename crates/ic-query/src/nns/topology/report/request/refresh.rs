use super::cache::{inventory_cache_request, subnet_catalog_cache_request};
use crate::{
    nns::NnsInventoryRefreshRequest,
    subnet_catalog::{CatalogSourceSelection, SubnetCatalogRefreshRequest},
};

pub(in crate::nns::topology::report) fn subnet_catalog_refresh_request(
    request: &super::model::NnsTopologyRefreshRequest,
) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest::new(
        subnet_catalog_cache_request(&request.read),
        CatalogSourceSelection::uncertified_query(request.read.source_endpoint.as_str()),
        request.read.now_unix_secs,
    )
    .with_dry_run(request.dry_run)
}

pub(in crate::nns::topology::report) fn inventory_refresh_request(
    request: &super::model::NnsTopologyRefreshRequest,
) -> NnsInventoryRefreshRequest {
    NnsInventoryRefreshRequest::new(
        inventory_cache_request(&request.read),
        request.read.source_endpoint.as_str(),
        request.read.now_unix_secs,
    )
    .with_dry_run(request.dry_run)
}
