use super::{
    NNS_NODE_CACHE_DIR, NNS_NODE_CACHE_FILE, NnsInventoryRefreshRequest, NnsNodeHostError,
    source::NnsNodeSource,
};
use crate::nns::NnsInventoryRefreshReport;
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub async fn refresh_nns_node_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsInventoryRefreshReport, NnsNodeHostError> {
    refresh_nns_node_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_node_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsInventoryRefreshReport, NnsNodeHostError> {
    super::enforce_mainnet_network(&request.cache.network)?;
    refresh_nns_inventory_cache(
        request,
        NNS_NODE_CACHE_DIR,
        NNS_NODE_CACHE_FILE,
        async |source_request| source.fetch_node_list_report(source_request).await,
    )
    .await
}
