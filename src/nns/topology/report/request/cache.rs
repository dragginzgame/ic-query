use super::model::TopologyRequestParts;
use crate::{
    nns::data_center::report::NnsDataCenterCacheRequest, nns::node::report::NnsNodeCacheRequest,
    nns::node_operator::report::NnsNodeOperatorCacheRequest,
    nns::node_provider::report::NnsNodeProviderCacheRequest,
    subnet_catalog::SubnetCatalogCacheRequest,
};

macro_rules! cache_request {
    ($name:ident, $request:path) => {
        pub(in crate::nns::topology::report) fn $name(
            request: &impl TopologyRequestParts,
        ) -> $request {
            $request {
                icp_root: request.icp_root().to_path_buf(),
                network: request.network().to_string(),
            }
        }
    };
}

cache_request!(subnet_catalog_cache_request, SubnetCatalogCacheRequest);
cache_request!(node_cache_request, NnsNodeCacheRequest);
cache_request!(node_provider_cache_request, NnsNodeProviderCacheRequest);
cache_request!(node_operator_cache_request, NnsNodeOperatorCacheRequest);
cache_request!(data_center_cache_request, NnsDataCenterCacheRequest);
