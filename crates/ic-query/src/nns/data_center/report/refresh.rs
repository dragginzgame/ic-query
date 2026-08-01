use super::{
    NNS_DATA_CENTER_CACHE_DIR, NNS_DATA_CENTER_CACHE_FILE,
    NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION, NnsDataCenterHostError, NnsDataCenterListReport,
    NnsDataCenterRefreshReport, NnsInventoryRefreshRequest,
    source::{NnsDataCenterSource, fetch_nns_data_center_list_report_with_source},
};
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

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
    let (report, write_result) = refresh_nns_inventory_cache(
        request,
        NNS_DATA_CENTER_CACHE_DIR,
        NNS_DATA_CENTER_CACHE_FILE,
        |network, source_endpoint, now_unix_secs| {
            fetch_nns_data_center_list_report_with_source(
                network,
                source_endpoint,
                now_unix_secs,
                source,
            )
        },
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
