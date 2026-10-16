use super::{
    NNS_NODE_OPERATOR_CACHE_DIR, NNS_NODE_OPERATOR_CACHE_FILE, NnsInventoryRefreshRequest,
    NnsNodeOperatorHostError, NnsNodeOperatorListReport,
    source::{NnsNodeOperatorSource, fetch_nns_node_operator_list_report_with_source},
};
use crate::nns::NnsInventoryRefreshReport;
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub async fn refresh_nns_node_operator_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsInventoryRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_node_operator_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsInventoryRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_cache_with_source(request, source)
        .await
        .map(|(_, report)| report)
}

pub(super) async fn refresh_nns_node_operator_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<(NnsNodeOperatorListReport, NnsInventoryRefreshReport), NnsNodeOperatorHostError> {
    refresh_nns_inventory_cache(
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
    .await
}
