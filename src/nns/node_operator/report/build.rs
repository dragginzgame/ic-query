use super::{
    DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS, NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION,
    NnsNodeOperatorHostError, NnsNodeOperatorInfoReport, NnsNodeOperatorInfoRequest,
    NnsNodeOperatorListReport, NnsNodeOperatorListRequest, NnsNodeOperatorRefreshRequest,
    cache::load_cached_nns_node_operator_report,
    refresh::refresh_nns_node_operator_cache_with_source,
    resolve::resolve_node_operator,
    source::{LiveNnsNodeOperatorSource, NnsNodeOperatorSource},
};
use crate::{cache_file::announce_cache_refresh, nns::leaf::NnsLeafHostCacheError};

pub fn build_nns_node_operator_list_report(
    request: &NnsNodeOperatorListRequest,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_list_report_with_source(request, &LiveNnsNodeOperatorSource)
}

pub fn build_nns_node_operator_info_report(
    request: &NnsNodeOperatorInfoRequest,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_info_report_with_source(request, &LiveNnsNodeOperatorSource)
}

pub(super) fn build_nns_node_operator_list_report_with_source(
    request: &NnsNodeOperatorListRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    match load_cached_nns_node_operator_report(&request.cache) {
        Ok(cached) => Ok(cached.report),
        Err(NnsNodeOperatorHostError::Cache(NnsLeafHostCacheError::MissingCache {
            path, ..
        })) => {
            announce_cache_refresh("node-operator", &path, &request.source_endpoint);
            let refresh_request = NnsNodeOperatorRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            let (report, _) =
                refresh_nns_node_operator_cache_with_source(&refresh_request, source)?;
            Ok(report)
        }
        Err(err) => Err(err),
    }
}

fn build_nns_node_operator_info_report_with_source(
    request: &NnsNodeOperatorInfoRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    let list_request = NnsNodeOperatorListRequest {
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
