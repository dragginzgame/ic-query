use super::{
    NNS_DATA_CENTER_CACHE_DIR, NNS_DATA_CENTER_CACHE_FILE, NnsDataCenterHostError,
    NnsInventoryRefreshRequest, source::NnsDataCenterSource,
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
    super::enforce_mainnet_network(&request.cache.network)?;
    refresh_nns_inventory_cache(
        request,
        NNS_DATA_CENTER_CACHE_DIR,
        NNS_DATA_CENTER_CACHE_FILE,
        async |source_request| source.fetch_data_center_list_report(source_request).await,
    )
    .await
}
