use super::{
    DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS, NNS_NODE_INFO_REPORT_SCHEMA_VERSION,
    NnsInventoryInfoRequest, NnsNodeHostError, NnsNodeInfoReport, NnsNodeListFilters,
    NnsNodeListReport, NnsNodeListRequest,
    cache::{load_cached_nns_node_report, nns_node_cache_path},
    filters::filter_node_list_report,
    refresh::refresh_nns_node_cache_with_source,
    resolve::resolve_node,
    source::NnsNodeSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};

pub fn build_nns_node_list_report(
    request: &NnsNodeListRequest,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    build_nns_node_list_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_node_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsNodeInfoReport, NnsNodeHostError> {
    build_nns_node_info_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_node_list_report_with_source(
    request: &NnsNodeListRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    let report = load_or_refresh_nns_inventory_report(
        request,
        nns_node_cache_path(&request.cache.cache_root, &request.cache.network),
        DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS,
        |cache| load_cached_nns_node_report(cache).map(|cached| cached.report),
        |refresh_request| refresh_nns_node_cache_with_source(refresh_request, source).map(|_| ()),
    )?;
    Ok(filter_node_list_report(report, &request.filters))
}

pub fn build_nns_node_info_report_with_source(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeInfoReport, NnsNodeHostError> {
    let list_request = NnsNodeListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
        filters: NnsNodeListFilters::default(),
    };
    let report = build_nns_node_list_report_with_source(&list_request, source)?;
    let (node, resolved_from) = resolve_node(&report, &request.input)?;
    Ok(NnsNodeInfoReport {
        schema_version: NNS_NODE_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        network: report.network,
        registry_canister_id: report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        node_principal: node.node_principal,
        node_operator_principal: node.node_operator_principal,
        node_provider_principal: node.node_provider_principal,
        subnet_principal: node.subnet_principal,
        subnet_kind: node.subnet_kind,
        data_center_id: node.data_center_id,
    })
}
