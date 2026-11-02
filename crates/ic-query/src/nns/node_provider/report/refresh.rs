use super::{
    NNS_NODE_PROVIDER_CACHE_DIR, NNS_NODE_PROVIDER_CACHE_FILE, NnsInventoryError,
    NnsInventoryRefreshRequest, source::NnsNodeProviderSource,
};
use crate::nns::NnsInventoryRefreshReport;
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub async fn refresh_nns_node_provider_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsInventoryRefreshReport, NnsInventoryError> {
    refresh_nns_node_provider_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_node_provider_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsInventoryRefreshReport, NnsInventoryError> {
    super::enforce_mainnet_network(&request.cache.network)?;
    refresh_nns_inventory_cache(
        request,
        NNS_NODE_PROVIDER_CACHE_DIR,
        NNS_NODE_PROVIDER_CACHE_FILE,
        async |source_request| source.fetch_node_provider_list_report(source_request).await,
    )
    .await
}
