use super::{
    NNS_NODE_CACHE_DIR, NNS_NODE_CACHE_FILE, NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
    NnsNodeCacheRequest, NnsNodeHostError, NnsNodeListReport, enforce_mainnet_network,
};
use crate::{cache_file::CachedJsonReport, nns::leaf::load_nns_leaf_json_cache};

pub(super) fn load_cached_nns_node_report(
    request: &NnsNodeCacheRequest,
) -> Result<CachedJsonReport<NnsNodeListReport>, NnsNodeHostError> {
    enforce_mainnet_network(&request.network)?;
    load_nns_leaf_json_cache(
        request,
        NNS_NODE_CACHE_DIR,
        NNS_NODE_CACHE_FILE,
        NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
    )
    .map_err(Into::into)
}
