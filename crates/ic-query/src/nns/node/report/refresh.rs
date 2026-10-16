use super::{
    NNS_NODE_CACHE_DIR, NNS_NODE_CACHE_FILE, NnsInventoryRefreshRequest, NnsNodeHostError,
    NnsNodeListReport,
    source::{NnsNodeSource, fetch_nns_node_list_report_with_source},
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
    refresh_nns_node_cache_with_source(request, source)
        .await
        .map(|(_, report)| report)
}

pub(super) async fn refresh_nns_node_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<(NnsNodeListReport, NnsInventoryRefreshReport), NnsNodeHostError> {
    refresh_nns_inventory_cache(
        request,
        NNS_NODE_CACHE_DIR,
        NNS_NODE_CACHE_FILE,
        async |network, source_endpoint, now_unix_secs| {
            fetch_nns_node_list_report_with_source(network, source_endpoint, now_unix_secs, source)
                .await
        },
    )
    .await
}
