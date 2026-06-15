use super::{
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterCacheErrors,
    NnsDataCenterCacheRequest, NnsDataCenterHostError, NnsDataCenterListReport,
    enforce_mainnet_network, nns_data_center_cache_path,
};
use crate::cache_file::{CachedJsonReport, LoadJsonCacheRequest, load_json_cache};

pub(super) fn load_cached_nns_data_center_report(
    request: &NnsDataCenterCacheRequest,
) -> Result<CachedJsonReport<NnsDataCenterListReport>, NnsDataCenterHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_data_center_cache_path(&request.icp_root, &request.network);
    load_json_cache(
        LoadJsonCacheRequest {
            path,
            network: &request.network,
            expected_schema_version: NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
        },
        NnsDataCenterCacheErrors,
    )
}
