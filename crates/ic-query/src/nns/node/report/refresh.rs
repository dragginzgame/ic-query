use super::{
    NNS_NODE_CACHE_DIR, NNS_NODE_CACHE_FILE, NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION,
    NnsNodeHostError, NnsNodeListReport, NnsNodeRefreshReport, NnsNodeRefreshRequest,
    enforce_mainnet_network,
    source::{NnsNodeSource, fetch_nns_node_list_report_with_source},
};
use crate::nns::{LiveNnsSource, leaf::write_nns_leaf_json_refresh_cache};

pub fn refresh_nns_node_report(
    request: &NnsNodeRefreshRequest,
) -> Result<NnsNodeRefreshReport, NnsNodeHostError> {
    refresh_nns_node_report_with_source(request, &LiveNnsSource)
}

pub fn refresh_nns_node_report_with_source(
    request: &NnsNodeRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeRefreshReport, NnsNodeHostError> {
    refresh_nns_node_cache_with_source(request, source).map(|(_, report)| report)
}

pub(super) fn refresh_nns_node_cache_with_source(
    request: &NnsNodeRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<(NnsNodeListReport, NnsNodeRefreshReport), NnsNodeHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let report = fetch_nns_node_list_report_with_source(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let write_result = write_nns_leaf_json_refresh_cache(
        request,
        NNS_NODE_CACHE_DIR,
        NNS_NODE_CACHE_FILE,
        &report,
    )?;
    let refresh_report = nns_leaf_refresh_report!(
        NnsNodeRefreshReport,
        NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION,
        request,
        report,
        write_result,
        node_count,
    );
    Ok((report, refresh_report))
}
