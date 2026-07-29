use super::{
    DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS, NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION,
    NnsNodeOperatorHostError, NnsNodeOperatorInfoReport, NnsNodeOperatorInfoRequest,
    NnsNodeOperatorListReport, NnsNodeOperatorListRequest, NnsNodeOperatorRefreshRequest,
    cache::load_cached_nns_node_operator_report,
    refresh::refresh_nns_node_operator_cache_with_source,
    resolve::resolve_node_operator,
    source::{LiveNnsNodeOperatorSource, NnsNodeOperatorSource},
};
use crate::{HostCacheError, cache_file::load_or_refresh_missing_cache};

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

pub fn build_nns_node_operator_list_report_with_source(
    request: &NnsNodeOperatorListRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    load_or_refresh_missing_cache(
        || load_cached_nns_node_operator_report(&request.cache).map(|cached| cached.report),
        |err| match err {
            NnsNodeOperatorHostError::Cache(HostCacheError::MissingCache { path, .. }) => Ok(path),
            err => Err(err),
        },
        |_| {
            let refresh_request = NnsNodeOperatorRefreshRequest::new(
                request.cache.clone(),
                request.source_endpoint.clone(),
                request.now_unix_secs,
                DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
            );
            refresh_nns_node_operator_cache_with_source(&refresh_request, source).map(|_| ())
        },
    )
}

pub fn build_nns_node_operator_info_report_with_source(
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
