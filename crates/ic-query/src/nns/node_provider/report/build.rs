use super::{
    DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS, NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION,
    NnsNodeProviderHostError, NnsNodeProviderInfoReport, NnsNodeProviderInfoRequest,
    NnsNodeProviderListReport, NnsNodeProviderListRequest, NnsNodeProviderRefreshRequest,
    cache::load_cached_nns_node_provider_report,
    refresh::refresh_nns_node_provider_cache_with_source,
    resolve::resolve_node_provider,
    source::{LiveNnsNodeProviderSource, NnsNodeProviderSource},
};
use crate::{cache_file::load_or_refresh_missing_cache, nns::leaf::NnsLeafHostCacheError};

pub fn build_nns_node_provider_list_report(
    request: &NnsNodeProviderListRequest,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    build_nns_node_provider_list_report_with_source(request, &LiveNnsNodeProviderSource)
}

pub fn build_nns_node_provider_info_report(
    request: &NnsNodeProviderInfoRequest,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    build_nns_node_provider_info_report_with_source(request, &LiveNnsNodeProviderSource)
}

pub(super) fn build_nns_node_provider_list_report_with_source(
    request: &NnsNodeProviderListRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    load_or_refresh_missing_cache(
        "node-provider",
        &request.source_endpoint,
        || load_cached_nns_node_provider_report(&request.cache).map(|cached| cached.report),
        || {
            let refresh_request = NnsNodeProviderRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            refresh_nns_node_provider_cache_with_source(&refresh_request, source).map(|_| ())
        },
        |err| match err {
            NnsNodeProviderHostError::Cache(NnsLeafHostCacheError::MissingCache {
                path, ..
            }) => Ok(path),
            err => Err(err),
        },
    )
}

pub(super) fn build_nns_node_provider_info_report_with_source(
    request: &NnsNodeProviderInfoRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    let list_request = NnsNodeProviderListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let report = build_nns_node_provider_list_report_with_source(&list_request, source)?;
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
