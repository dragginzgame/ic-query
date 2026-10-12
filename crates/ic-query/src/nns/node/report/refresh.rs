use super::{
    NNS_NODE_CACHE_DIR, NNS_NODE_CACHE_FILE, NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION,
    NnsInventoryRefreshRequest, NnsNodeHostError, NnsNodeListReport, NnsNodeRefreshReport,
    source::{NnsNodeSource, fetch_nns_node_list_report_with_source},
};
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub async fn refresh_nns_node_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsNodeRefreshReport, NnsNodeHostError> {
    refresh_nns_node_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_node_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeRefreshReport, NnsNodeHostError> {
    refresh_nns_node_cache_with_source(request, source)
        .await
        .map(|(_, report)| report)
}

pub(super) async fn refresh_nns_node_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<(NnsNodeListReport, NnsNodeRefreshReport), NnsNodeHostError> {
    let (report, write_result) = refresh_nns_inventory_cache(
        request,
        NNS_NODE_CACHE_DIR,
        NNS_NODE_CACHE_FILE,
        async |network, source_endpoint, now_unix_secs| {
            fetch_nns_node_list_report_with_source(network, source_endpoint, now_unix_secs, source)
                .await
        },
    )
    .await?;
    let refresh_report = nns_leaf_refresh_report!(
        NnsNodeRefreshReport,
        NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION,
        request,
        report,
        write_result,
        node_count,
    );
    Ok((report, refresh_report))
}
