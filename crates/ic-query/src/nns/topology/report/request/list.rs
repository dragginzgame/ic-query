use super::{
    cache::{
        data_center_cache_request, node_cache_request, node_operator_cache_request,
        node_provider_cache_request, subnet_catalog_cache_request,
    },
    model::TopologyRequestParts,
};
use crate::{
    nns::data_center::report::NnsDataCenterListRequest,
    nns::node::report::{NnsNodeListFilters, NnsNodeListRequest},
    nns::node_operator::report::NnsNodeOperatorListRequest,
    nns::node_provider::report::NnsNodeProviderListRequest,
    subnet_catalog::{DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogFilters, SubnetCatalogListRequest},
};

pub(in crate::nns::topology::report) fn subnet_catalog_list_request(
    request: &impl TopologyRequestParts,
) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest {
        cache: subnet_catalog_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
        filters: SubnetCatalogFilters::default(),
        show_ranges: false,
        range_limit: 1,
        range_offset: 0,
    }
}

pub(in crate::nns::topology::report) fn node_list_request(
    request: &impl TopologyRequestParts,
) -> NnsNodeListRequest {
    NnsNodeListRequest {
        cache: node_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        filters: NnsNodeListFilters::default(),
    }
}

macro_rules! component_list_request {
    ($name:ident, $request:path, $cache_request:ident) => {
        pub(in crate::nns::topology::report) fn $name(
            request: &impl TopologyRequestParts,
        ) -> $request {
            $request {
                cache: $cache_request(request),
                source_endpoint: request.source_endpoint().to_string(),
                now_unix_secs: request.now_unix_secs(),
            }
        }
    };
}

component_list_request!(
    node_provider_list_request,
    NnsNodeProviderListRequest,
    node_provider_cache_request
);
component_list_request!(
    node_operator_list_request,
    NnsNodeOperatorListRequest,
    node_operator_cache_request
);
component_list_request!(
    data_center_list_request,
    NnsDataCenterListRequest,
    data_center_cache_request
);
