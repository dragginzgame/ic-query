use super::{
    NNS_DATA_CENTER_CACHE_DIR, NNS_DATA_CENTER_CACHE_FILE,
    NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION, NnsDataCenterHostError, NnsDataCenterListReport,
    NnsDataCenterRefreshReport, NnsInventoryRefreshRequest, enforce_mainnet_network,
    source::{NnsDataCenterSource, fetch_nns_data_center_list_report_with_source},
};
use crate::nns::{LiveNnsSource, leaf::write_nns_leaf_json_refresh_cache};

pub fn refresh_nns_data_center_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsDataCenterRefreshReport, NnsDataCenterHostError> {
    refresh_nns_data_center_report_with_source(request, &LiveNnsSource)
}

pub fn refresh_nns_data_center_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterRefreshReport, NnsDataCenterHostError> {
    refresh_nns_data_center_cache_with_source(request, source).map(|(_, report)| report)
}

pub(super) fn refresh_nns_data_center_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<(NnsDataCenterListReport, NnsDataCenterRefreshReport), NnsDataCenterHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let report = fetch_nns_data_center_list_report_with_source(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let write_result = write_nns_leaf_json_refresh_cache(
        request,
        NNS_DATA_CENTER_CACHE_DIR,
        NNS_DATA_CENTER_CACHE_FILE,
        &report,
    )?;
    let refresh_report = nns_leaf_refresh_report!(
        NnsDataCenterRefreshReport,
        NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION,
        request,
        report,
        write_result,
        data_center_count,
    );
    Ok((report, refresh_report))
}
