use super::{
    NNS_NODE_OPERATOR_CACHE_DIR, NNS_NODE_OPERATOR_CACHE_FILE,
    NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION, NnsNodeOperatorHostError,
    NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport, NnsNodeOperatorRefreshRequest,
    enforce_mainnet_network,
    source::{
        LiveNnsNodeOperatorSource, NnsNodeOperatorSource,
        fetch_nns_node_operator_list_report_with_source,
    },
};
use crate::nns::leaf::write_nns_leaf_json_refresh_cache;

pub fn refresh_nns_node_operator_report(
    request: &NnsNodeOperatorRefreshRequest,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_report_with_source(request, &LiveNnsNodeOperatorSource)
}

pub fn refresh_nns_node_operator_report_with_source(
    request: &NnsNodeOperatorRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_cache_with_source(request, source).map(|(_, report)| report)
}

pub(super) fn refresh_nns_node_operator_cache_with_source(
    request: &NnsNodeOperatorRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<(NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport), NnsNodeOperatorHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let report = fetch_nns_node_operator_list_report_with_source(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let write_result = write_nns_leaf_json_refresh_cache(
        request,
        NNS_NODE_OPERATOR_CACHE_DIR,
        NNS_NODE_OPERATOR_CACHE_FILE,
        &report,
    )?;
    let refresh_report = NnsNodeOperatorRefreshReport {
        schema_version: NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION,
        network: report.network.clone(),
        cache_path: write_result.cache_path,
        refresh_lock_path: write_result.refresh_lock_path,
        output_path: write_result.output_path,
        registry_canister_id: report.registry_canister_id.clone(),
        registry_version: report.registry_version,
        fetched_at: report.fetched_at.clone(),
        source_endpoint: report.source_endpoint.clone(),
        fetched_by: report.fetched_by.clone(),
        dry_run: request.dry_run,
        wrote_cache: write_result.wrote_cache,
        replaced_existing_cache: write_result.replaced_existing_cache,
        node_operator_count: report.node_operator_count,
    };
    Ok((report, refresh_report))
}
