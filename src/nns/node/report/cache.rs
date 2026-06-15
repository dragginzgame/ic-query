use super::{
    NNS_NODE_LIST_REPORT_SCHEMA_VERSION, NnsNodeCacheErrors, NnsNodeCacheRequest, NnsNodeHostError,
    NnsNodeListReport, enforce_mainnet_network, nns_node_cache_path,
};
use crate::cache_file::{CachedJsonReport, LoadJsonCacheRequest, load_json_cache};

pub(super) fn load_cached_nns_node_report(
    request: &NnsNodeCacheRequest,
) -> Result<CachedJsonReport<NnsNodeListReport>, NnsNodeHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_node_cache_path(&request.icp_root, &request.network);
    load_json_cache(
        LoadJsonCacheRequest {
            path,
            network: &request.network,
            expected_schema_version: NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
        },
        NnsNodeCacheErrors,
    )
}
