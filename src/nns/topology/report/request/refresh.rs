use super::{
    cache::{
        data_center_cache_request, node_cache_request, node_operator_cache_request,
        node_provider_cache_request, subnet_catalog_cache_request,
    },
    model::TopologyRefreshParts,
};
use crate::{
    nns::data_center::report::NnsDataCenterRefreshRequest,
    nns::node::report::NnsNodeRefreshRequest,
    nns::node_operator::report::NnsNodeOperatorRefreshRequest,
    nns::node_provider::report::NnsNodeProviderRefreshRequest,
    subnet_catalog::SubnetCatalogRefreshRequest,
};

pub(in crate::nns::topology::report) fn subnet_catalog_refresh_request(
    request: &impl TopologyRefreshParts,
) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest {
        cache: subnet_catalog_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        lock_stale_after_seconds: request.lock_stale_after_seconds(),
        dry_run: request.dry_run(),
        output_path: None,
    }
}

pub(in crate::nns::topology::report) fn node_refresh_request(
    request: &impl TopologyRefreshParts,
) -> NnsNodeRefreshRequest {
    NnsNodeRefreshRequest {
        cache: node_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        lock_stale_after_seconds: request.lock_stale_after_seconds(),
        dry_run: request.dry_run(),
        output_path: None,
    }
}

macro_rules! component_refresh_request {
    ($name:ident, $request:path, $cache_request:ident) => {
        pub(in crate::nns::topology::report) fn $name(
            request: &impl TopologyRefreshParts,
        ) -> $request {
            $request {
                cache: $cache_request(request),
                source_endpoint: request.source_endpoint().to_string(),
                now_unix_secs: request.now_unix_secs(),
                lock_stale_after_seconds: request.lock_stale_after_seconds(),
                dry_run: request.dry_run(),
                output_path: None,
            }
        }
    };
}

component_refresh_request!(
    node_provider_refresh_request,
    NnsNodeProviderRefreshRequest,
    node_provider_cache_request
);
component_refresh_request!(
    node_operator_refresh_request,
    NnsNodeOperatorRefreshRequest,
    node_operator_cache_request
);
component_refresh_request!(
    data_center_refresh_request,
    NnsDataCenterRefreshRequest,
    data_center_cache_request
);
