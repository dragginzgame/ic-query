use super::{
    NNS_NODE_OPERATOR_CACHE_DIR, NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION, NnsNodeOperatorHostError,
    NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport, NnsNodeOperatorRefreshRequest,
    enforce_mainnet_network,
    source::{
        LiveNnsNodeOperatorSource, NnsNodeOperatorSource,
        fetch_nns_node_operator_list_report_with_source,
    },
};
use crate::nns::leaf::write_nns_leaf_json_refresh_cache;

pub fn refresh_nns_node_operator_report(
    request: &NnsNodeOperatorRefreshRequest,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_report_with_source(request, &LiveNnsNodeOperatorSource)
}

pub fn refresh_nns_node_operator_report_with_source(
    request: &NnsNodeOperatorRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_cache_with_source(request, source).map(|(_, report)| report)
}

pub(super) fn refresh_nns_node_operator_cache_with_source(
    request: &NnsNodeOperatorRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<(NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport), NnsNodeOperatorHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let report = fetch_nns_node_operator_list_report_with_source(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let write_result = write_nns_leaf_json_refresh_cache(
        request,
        NNS_NODE_OPERATOR_CACHE_DIR,
        NNS_NODE_OPERATOR_CACHE_FILE,
        &report,
    )?;
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
