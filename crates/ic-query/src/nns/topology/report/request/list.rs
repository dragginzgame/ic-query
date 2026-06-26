use super::{
    cache::{
        data_center_cache_request, node_cache_request, node_operator_cache_request,
        node_provider_cache_request, subnet_catalog_cache_request,
    },
    model::TopologyRequestParts,
};
use crate::{
    nns::data_center::report::NnsDataCenterListRequest,
    nns::node::report::NnsNodeListRequest,
    nns::node_operator::report::NnsNodeOperatorListRequest,
    nns::node_provider::report::NnsNodeProviderListRequest,
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
        node_cache_request(request),
        request.source_endpoint(),
        request.now_unix_secs(),
    )
}

macro_rules! component_list_request {
    ($name:ident, $request:path, $cache_request:ident) => {
        pub(in crate::nns::topology::report) fn $name(
            request: &impl TopologyRequestParts,
        ) -> $request {
            <$request>::new(
                $cache_request(request),
                request.source_endpoint(),
                request.now_unix_secs(),
            )
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
