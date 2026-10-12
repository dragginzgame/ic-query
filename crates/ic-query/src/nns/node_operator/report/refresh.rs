use super::{
    NNS_NODE_OPERATOR_CACHE_DIR, NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION, NnsInventoryRefreshRequest,
    NnsNodeOperatorHostError, NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport,
    source::{NnsNodeOperatorSource, fetch_nns_node_operator_list_report_with_source},
};
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub async fn refresh_nns_node_operator_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_node_operator_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_cache_with_source(request, source)
        .await
        .map(|(_, report)| report)
}

pub(super) async fn refresh_nns_node_operator_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<(NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport), NnsNodeOperatorHostError> {
    let (report, write_result) = refresh_nns_inventory_cache(
        request,
        NNS_NODE_OPERATOR_CACHE_DIR,
        NNS_NODE_OPERATOR_CACHE_FILE,
        async |network, source_endpoint, now_unix_secs| {
            fetch_nns_node_operator_list_report_with_source(
                network,
                source_endpoint,
                now_unix_secs,
                source,
            )
            .await
        },
    )
    .await?;
    let refresh_report = nns_leaf_refresh_report!(
        NnsNodeOperatorRefreshReport,
        NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION,
        request,
        report,
        write_result,
        node_operator_count,
    );
    Ok((report, refresh_report))
}
