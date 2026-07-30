use super::{
    cache::{inventory_cache_request, subnet_catalog_cache_request},
    model::TopologyRefreshParts,
};
use crate::{nns::NnsInventoryRefreshRequest, subnet_catalog::SubnetCatalogRefreshRequest};

pub(in crate::nns::topology::report) fn subnet_catalog_refresh_request(
    request: &impl TopologyRefreshParts,
) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest::new(
        subnet_catalog_cache_request(request),
        request.source_endpoint(),
        request.now_unix_secs(),
        request.lock_stale_after_seconds(),
    )
    .with_dry_run(request.dry_run())
}

pub(in crate::nns::topology::report) fn inventory_refresh_request(
    request: &impl TopologyRefreshParts,
) -> NnsInventoryRefreshRequest {
    NnsInventoryRefreshRequest::new(
        inventory_cache_request(request),
        request.source_endpoint(),
        request.now_unix_secs(),
        request.lock_stale_after_seconds(),
    )
    .with_dry_run(request.dry_run())
}
