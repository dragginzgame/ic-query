use super::model::TopologyRequestParts;
use crate::{
    nns::{
        data_center::report::NnsDataCenterCacheRequest, leaf::NnsLeafCacheRequest,
        node::report::NnsNodeCacheRequest, node_operator::report::NnsNodeOperatorCacheRequest,
        node_provider::report::NnsNodeProviderCacheRequest,
    },
    subnet_catalog::SubnetCatalogCacheRequest,
};

macro_rules! nns_leaf_cache_request {
    ($name:ident, $request:path) => {
        pub(in crate::nns::topology::report) fn $name(
            request: &impl TopologyRequestParts,
        ) -> $request {
            <$request as NnsLeafCacheRequest>::from_root_network(
                request.icp_root(),
                request.network(),
            )
        }
    };
}

pub(in crate::nns::topology::report) fn subnet_catalog_cache_request(
    request: &impl TopologyRequestParts,
) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest::new(request.icp_root(), request.network())
}

nns_leaf_cache_request!(node_cache_request, NnsNodeCacheRequest);
nns_leaf_cache_request!(node_provider_cache_request, NnsNodeProviderCacheRequest);
nns_leaf_cache_request!(node_operator_cache_request, NnsNodeOperatorCacheRequest);
nns_leaf_cache_request!(data_center_cache_request, NnsDataCenterCacheRequest);
