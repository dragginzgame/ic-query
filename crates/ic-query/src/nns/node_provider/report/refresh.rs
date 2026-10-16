use super::{
    NNS_NODE_PROVIDER_CACHE_DIR, NNS_NODE_PROVIDER_CACHE_FILE, NnsInventoryRefreshRequest,
    NnsNodeProviderHostError, NnsNodeProviderListReport,
    source::{NnsNodeProviderSource, fetch_nns_node_provider_list_report_with_source},
};
use crate::nns::NnsInventoryRefreshReport;
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub async fn refresh_nns_node_provider_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsInventoryRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_node_provider_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsInventoryRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_cache_with_source(request, source)
        .await
        .map(|(_, report)| report)
}

pub(super) async fn refresh_nns_node_provider_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<(NnsNodeProviderListReport, NnsInventoryRefreshReport), NnsNodeProviderHostError> {
    refresh_nns_inventory_cache(
        request,
        NNS_NODE_PROVIDER_CACHE_DIR,
        NNS_NODE_PROVIDER_CACHE_FILE,
        async |network, source_endpoint, now_unix_secs| {
            fetch_nns_node_provider_list_report_with_source(
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
