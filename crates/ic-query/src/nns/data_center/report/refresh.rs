use super::{
    NNS_DATA_CENTER_CACHE_DIR, NNS_DATA_CENTER_CACHE_FILE, NnsDataCenterHostError,
    NnsDataCenterListReport, NnsInventoryRefreshRequest,
    source::{NnsDataCenterSource, fetch_nns_data_center_list_report_with_source},
};
use crate::nns::NnsInventoryRefreshReport;
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub async fn refresh_nns_data_center_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsInventoryRefreshReport, NnsDataCenterHostError> {
    refresh_nns_data_center_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_data_center_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsInventoryRefreshReport, NnsDataCenterHostError> {
    refresh_nns_data_center_cache_with_source(request, source)
        .await
        .map(|(_, report)| report)
}

pub(super) async fn refresh_nns_data_center_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<(NnsDataCenterListReport, NnsInventoryRefreshReport), NnsDataCenterHostError> {
    refresh_nns_inventory_cache(
        request,
        NNS_DATA_CENTER_CACHE_DIR,
        NNS_DATA_CENTER_CACHE_FILE,
        async |network, source_endpoint, now_unix_secs| {
            fetch_nns_data_center_list_report_with_source(
                network,
                source_endpoint,
                now_unix_secs,
                source,
            )
            .await
        },
    )
    .await
}
