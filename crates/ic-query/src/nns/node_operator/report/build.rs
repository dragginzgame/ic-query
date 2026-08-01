use super::{
    DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS, NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION,
    NnsInventoryInfoRequest, NnsInventoryListRequest, NnsNodeOperatorHostError,
    NnsNodeOperatorInfoReport, NnsNodeOperatorListReport,
    cache::load_cached_nns_node_operator_report,
    refresh::refresh_nns_node_operator_cache_with_source, resolve::resolve_node_operator,
    source::NnsNodeOperatorSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};

pub fn build_nns_node_operator_list_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_list_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_node_operator_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_info_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_node_operator_list_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    load_or_refresh_nns_inventory_report(
        request,
        DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
        |cache| load_cached_nns_node_operator_report(cache).map(|cached| cached.report),
        |refresh_request| {
            refresh_nns_node_operator_cache_with_source(refresh_request, source).map(|_| ())
        },
    )
}

pub fn build_nns_node_operator_info_report_with_source(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    let list_request = NnsInventoryListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let report = build_nns_node_operator_list_report_with_source(&list_request, source)?;
    let (operator, resolved_from) = resolve_node_operator(&report, &request.input)?;
    Ok(NnsNodeOperatorInfoReport {
        schema_version: NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        network: report.network,
        registry_canister_id: report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        node_operator_principal: operator.node_operator_principal,
        node_provider_principal: operator.node_provider_principal,
        node_allowance: operator.node_allowance,
        data_center_id: operator.data_center_id,
        node_count: operator.node_count,
    })
}
