use super::{
    NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsNodeOperatorHostError, NnsNodeOperatorInfoReport, NnsNodeOperatorListReport,
    cache::{load_cached_nns_node_operator_report, nns_node_operator_cache_path},
    refresh::refresh_nns_node_operator_report_with_source,
    resolve::resolve_node_operator,
    source::NnsNodeOperatorSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};
use crate::{QueryProgress, progress::IgnoreQueryProgress};

pub async fn build_nns_node_operator_list_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_list_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_node_operator_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_info_report_with_source(request, &LiveNnsSource::default()).await
}

/// Build the report while reporting authorized cache refreshes.
pub async fn build_nns_node_operator_list_report_with_progress(
    request: &NnsInventoryListRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_list_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        progress,
    )
    .await
}

pub async fn build_nns_node_operator_list_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_list_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_operator_list_report_with_source_and_progress(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeOperatorSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    load_or_refresh_nns_inventory_report(
        request,
        nns_node_operator_cache_path(&request.cache.cache_root, &request.cache.network),
        progress,
        |cache| load_cached_nns_node_operator_report(cache).map(|cached| cached.report),
        async |refresh_request| {
            refresh_nns_node_operator_report_with_source(refresh_request, source)
                .await
                .map(|_| ())
        },
    )
    .await
}

/// Build the report while reporting authorized cache refreshes.
pub async fn build_nns_node_operator_info_report_with_progress(
    request: &NnsInventoryInfoRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_info_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        progress,
    )
    .await
}

pub async fn build_nns_node_operator_info_report_with_source(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_info_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_operator_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeOperatorSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    let list_request = NnsInventoryListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let mut report = build_nns_node_operator_list_report_with_source_and_progress(
        &list_request,
        source,
        progress,
    )
    .await?;
    let (index, resolved_from) = resolve_node_operator(&report, &request.input)?;
    let operator = report.node_operators.swap_remove(index);
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
