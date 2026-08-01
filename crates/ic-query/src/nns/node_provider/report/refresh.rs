use super::{
    NNS_NODE_PROVIDER_CACHE_DIR, NNS_NODE_PROVIDER_CACHE_FILE,
    NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION, NnsInventoryRefreshRequest,
    NnsNodeProviderHostError, NnsNodeProviderListReport, NnsNodeProviderRefreshReport,
    source::{NnsNodeProviderSource, fetch_nns_node_provider_list_report_with_source},
};
use crate::nns::{LiveNnsSource, inventory::refresh_nns_inventory_cache};

pub fn refresh_nns_node_provider_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsNodeProviderRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_report_with_source(request, &LiveNnsSource)
}

pub fn refresh_nns_node_provider_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_cache_with_source(request, source).map(|(_, report)| report)
}

pub(super) fn refresh_nns_node_provider_cache_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<(NnsNodeProviderListReport, NnsNodeProviderRefreshReport), NnsNodeProviderHostError> {
    let (report, write_result) = refresh_nns_inventory_cache(
        request,
        NNS_NODE_PROVIDER_CACHE_DIR,
        NNS_NODE_PROVIDER_CACHE_FILE,
        |network, source_endpoint, now_unix_secs| {
            fetch_nns_node_provider_list_report_with_source(
                network,
                source_endpoint,
                now_unix_secs,
                source,
            )
        },
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
