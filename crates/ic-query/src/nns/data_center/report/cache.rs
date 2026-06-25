use super::{
    NNS_DATA_CENTER_CACHE_DIR, NNS_DATA_CENTER_CACHE_FILE,
    NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION, NnsDataCenterCacheRequest, NnsDataCenterHostError,
    NnsDataCenterListReport, enforce_mainnet_network,
};
use crate::{cache_file::CachedJsonReport, nns::leaf::load_nns_leaf_json_cache};

pub(super) fn load_cached_nns_data_center_report(
    request: &NnsDataCenterCacheRequest,
) -> Result<CachedJsonReport<NnsDataCenterListReport>, NnsDataCenterHostError> {
    enforce_mainnet_network(&request.network)?;
    load_nns_leaf_json_cache(
        request,
        NNS_DATA_CENTER_CACHE_DIR,
        NNS_DATA_CENTER_CACHE_FILE,
        NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
    )
    .map_err(Into::into)
}
