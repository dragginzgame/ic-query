use super::{
    NNS_NODE_PROVIDER_CACHE_DIR, NNS_NODE_PROVIDER_CACHE_FILE,
    NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION, NnsNodeProviderHostError,
    NnsNodeProviderListReport, NnsNodeProviderRefreshReport, NnsNodeProviderRefreshRequest,
    enforce_mainnet_network,
    source::{
        LiveNnsNodeProviderSource, NnsNodeProviderSource,
        fetch_nns_node_provider_list_report_with_source,
    },
};
use crate::nns::leaf::write_nns_leaf_json_refresh_cache;

pub fn refresh_nns_node_provider_report(
    request: &NnsNodeProviderRefreshRequest,
) -> Result<NnsNodeProviderRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_report_with_source(request, &LiveNnsNodeProviderSource)
}

pub fn refresh_nns_node_provider_report_with_source(
    request: &NnsNodeProviderRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_cache_with_source(request, source).map(|(_, report)| report)
}

pub(super) fn refresh_nns_node_provider_cache_with_source(
    request: &NnsNodeProviderRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<(NnsNodeProviderListReport, NnsNodeProviderRefreshReport), NnsNodeProviderHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let report = fetch_nns_node_provider_list_report_with_source(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let write_result = write_nns_leaf_json_refresh_cache(
        request,
        NNS_NODE_PROVIDER_CACHE_DIR,
        NNS_NODE_PROVIDER_CACHE_FILE,
        &report,
    )?;
    let refresh_report = NnsNodeProviderRefreshReport {
        schema_version: NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION,
        network: report.network.clone(),
        cache_path: write_result.cache_path,
        refresh_lock_path: write_result.refresh_lock_path,
        output_path: write_result.output_path,
        governance_canister_id: report.governance_canister_id.clone(),
        registry_canister_id: report.registry_canister_id.clone(),
        registry_version: report.registry_version,
        fetched_at: report.fetched_at.clone(),
        source_endpoint: report.source_endpoint.clone(),
        fetched_by: report.fetched_by.clone(),
        dry_run: request.dry_run,
        wrote_cache: write_result.wrote_cache,
        replaced_existing_cache: write_result.replaced_existing_cache,
        node_provider_count: report.node_provider_count,
    };
    Ok((report, refresh_report))
}
