use crate::nns::topology::report::{
    LiveNnsTopologySource, NnsTopologyHostError, NnsTopologySource, NnsTopologySummaryReport,
    NnsTopologySummaryRequest, enforce_mainnet_network, request::TopologyRequestParts,
    source::topology_source_request_from, summary::topology_summary_report_from_reports,
};

pub fn build_nns_topology_summary_report(
    request: &NnsTopologySummaryRequest,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    build_nns_topology_summary_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_summary_report_with_source(
    request: &NnsTopologySummaryRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let source_request = topology_source_request_from(request);
    let subnet_report = source.fetch_subnet_catalog_list_report(&source_request)?;
    let node_report = source.fetch_node_list_report(&source_request)?;
    let node_provider_report = source.fetch_node_provider_list_report(&source_request)?;
    let node_operator_report = source.fetch_node_operator_list_report(&source_request)?;
    let data_center_report = source.fetch_data_center_list_report(&source_request)?;

    Ok(topology_summary_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        subnet_report,
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ))
}
