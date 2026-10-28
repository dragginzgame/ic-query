use super::{
    NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsNodeProviderHostError, NnsNodeProviderInfoReport, NnsNodeProviderListReport,
    cache::{load_cached_nns_node_provider_report, nns_node_provider_cache_path},
    refresh::refresh_nns_node_provider_report_with_source,
    resolve::resolve_node_provider,
    source::NnsNodeProviderSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};
use crate::{QueryProgress, progress::IgnoreQueryProgress};

pub async fn build_nns_node_provider_list_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    build_nns_node_provider_list_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_node_provider_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    build_nns_node_provider_info_report_with_source(request, &LiveNnsSource::default()).await
}

/// Build the report while reporting authorized cache refreshes.
pub async fn build_nns_node_provider_list_report_with_progress(
    request: &NnsInventoryListRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    build_nns_node_provider_list_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        progress,
    )
    .await
}

pub async fn build_nns_node_provider_list_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    build_nns_node_provider_list_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_provider_list_report_with_source_and_progress(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeProviderSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    load_or_refresh_nns_inventory_report(
        request,
        nns_node_provider_cache_path(&request.cache.cache_root, &request.cache.network),
        progress,
        |cache| load_cached_nns_node_provider_report(cache).map(|cached| cached.report),
        async |refresh_request| {
            refresh_nns_node_provider_report_with_source(refresh_request, source)
                .await
                .map(|_| ())
        },
    )
    .await
}

/// Build the report while reporting authorized cache refreshes.
pub async fn build_nns_node_provider_info_report_with_progress(
    request: &NnsInventoryInfoRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    build_nns_node_provider_info_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        progress,
    )
    .await
}

pub async fn build_nns_node_provider_info_report_with_source(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    build_nns_node_provider_info_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_provider_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeProviderSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    let list_request = NnsInventoryListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let report = build_nns_node_provider_list_report_with_source_and_progress(
        &list_request,
        source,
        progress,
    )
    .await?;
    let (provider, resolved_from) = resolve_node_provider(&report, &request.input)?;
    Ok(NnsNodeProviderInfoReport {
        schema_version: NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        network: report.network,
        governance_canister_id: report.governance_canister_id,
        registry_canister_id: report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        node_provider_principal: provider.node_provider_principal,
        name: provider.name,
        node_count: provider.node_count,
        reward_account_hex: provider.reward_account_hex,
    })
}
