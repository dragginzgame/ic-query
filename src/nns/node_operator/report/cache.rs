use super::{
    NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION, NnsNodeOperatorCacheErrors,
    NnsNodeOperatorCacheRequest, NnsNodeOperatorHostError, NnsNodeOperatorListReport,
    enforce_mainnet_network, nns_node_operator_cache_path,
};
use crate::cache_file::{CachedJsonReport, LoadJsonCacheRequest, load_json_cache};

pub(super) fn load_cached_nns_node_operator_report(
    request: &NnsNodeOperatorCacheRequest,
) -> Result<CachedJsonReport<NnsNodeOperatorListReport>, NnsNodeOperatorHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_node_operator_cache_path(&request.icp_root, &request.network);
    load_json_cache(
        LoadJsonCacheRequest {
            path,
            network: &request.network,
            expected_schema_version: NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
        },
        NnsNodeOperatorCacheErrors,
    )
}
