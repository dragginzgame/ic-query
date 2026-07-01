use crate::nns::topology::report::{
    LiveNnsTopologySource, NnsTopologyHostError, NnsTopologyRefreshReport,
    NnsTopologyRefreshRequest, NnsTopologyRefreshSource, enforce_mainnet_network,
    refresh::{NnsTopologyRefreshComponentReports, topology_refresh_report_from_reports},
    request::{TopologyRefreshParts, TopologyRequestParts},
    source::topology_refresh_source_request_from,
};

pub fn refresh_nns_topology_report(
    request: &NnsTopologyRefreshRequest,
) -> Result<NnsTopologyRefreshReport, NnsTopologyHostError> {
    refresh_nns_topology_report_with_source(request, &LiveNnsTopologySource)
}

pub fn refresh_nns_topology_report_with_source(
    request: &NnsTopologyRefreshRequest,
    source: &dyn NnsTopologyRefreshSource,
) -> Result<NnsTopologyRefreshReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let source_request = topology_refresh_source_request_from(request);
    let subnet_report = source.refresh_subnet_catalog_report(&source_request)?;
    let node_report = source.refresh_node_report(&source_request)?;
    let node_provider_report = source.refresh_node_provider_report(&source_request)?;
    let node_operator_report = source.refresh_node_operator_report(&source_request)?;
    let data_center_report = source.refresh_data_center_report(&source_request)?;

    Ok(topology_refresh_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        request.dry_run(),
        NnsTopologyRefreshComponentReports {
            subnet: subnet_report,
            node: node_report,
            node_provider: node_provider_report,
            node_operator: node_operator_report,
            data_center: data_center_report,
        },
    ))
}
