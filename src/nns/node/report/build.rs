use super::{
    DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS, NNS_NODE_INFO_REPORT_SCHEMA_VERSION, NnsNodeHostError,
    NnsNodeInfoReport, NnsNodeInfoRequest, NnsNodeListFilters, NnsNodeListReport,
    NnsNodeListRequest, NnsNodeRefreshRequest,
    cache::load_cached_nns_node_report,
    filters::filter_node_list_report,
    refresh::refresh_nns_node_cache_with_source,
    resolve::resolve_node,
    source::{LiveNnsNodeSource, NnsNodeSource},
};
use crate::{cache_file::announce_cache_refresh, nns::leaf::NnsLeafHostCacheError};

pub fn build_nns_node_list_report(
    request: &NnsNodeListRequest,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    build_nns_node_list_report_with_source(request, &LiveNnsNodeSource)
}

pub fn build_nns_node_info_report(
    request: &NnsNodeInfoRequest,
) -> Result<NnsNodeInfoReport, NnsNodeHostError> {
    build_nns_node_info_report_with_source(request, &LiveNnsNodeSource)
}

pub(super) fn build_nns_node_list_report_with_source(
    request: &NnsNodeListRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    let report = match load_cached_nns_node_report(&request.cache) {
        Ok(cached) => cached.report,
        Err(NnsNodeHostError::Cache(NnsLeafHostCacheError::MissingCache { path, .. })) => {
            announce_cache_refresh("node", &path, &request.source_endpoint);
            let refresh_request = NnsNodeRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            let (report, _) = refresh_nns_node_cache_with_source(&refresh_request, source)?;
            report
        }
        Err(err) => return Err(err),
    };
    Ok(filter_node_list_report(report, &request.filters))
}

fn build_nns_node_info_report_with_source(
    request: &NnsNodeInfoRequest,
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
