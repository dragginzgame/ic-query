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
    let refresh_report = nns_leaf_refresh_report!(
        NnsNodeProviderRefreshReport,
        NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION,
        request,
        report,
        write_result,
        node_provider_count,
        report.governance_canister_id.clone(),
    );
    Ok((report, refresh_report))
}
