use super::{
    NNS_NODE_PROVIDER_CACHE_DIR, NNS_NODE_PROVIDER_CACHE_FILE,
    NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION, NnsNodeProviderCacheRequest,
    NnsNodeProviderHostError, NnsNodeProviderListReport, enforce_mainnet_network,
};
use crate::{cache_file::CachedJsonReport, nns::leaf::load_nns_leaf_json_cache};

pub(super) fn load_cached_nns_node_provider_report(
    request: &NnsNodeProviderCacheRequest,
) -> Result<CachedJsonReport<NnsNodeProviderListReport>, NnsNodeProviderHostError> {
    enforce_mainnet_network(&request.network)?;
    load_nns_leaf_json_cache(
        request,
        NNS_NODE_PROVIDER_CACHE_DIR,
        NNS_NODE_PROVIDER_CACHE_FILE,
        NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
    )
    .map_err(Into::into)
}
