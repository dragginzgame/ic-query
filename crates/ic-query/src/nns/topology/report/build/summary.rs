use super::fetch_topology_inventory_reports;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NnsTopologyHostError, NnsTopologyReadRequest, NnsTopologySource, NnsTopologySummaryReport,
        enforce_mainnet_network, request::TopologyRequestParts,
        source::topology_source_request_from, summary::topology_summary_report_from_reports,
    },
};

pub fn build_nns_topology_summary_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    build_nns_topology_summary_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_topology_summary_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let source_request = topology_source_request_from(request);
    let subnet_report = source.fetch_subnet_catalog_list_report(&source_request)?;
    let reports = fetch_topology_inventory_reports(&source_request, source)?;

    Ok(topology_summary_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        subnet_report,
        reports.node,
        reports.node_provider,
        reports.node_operator,
        reports.data_center,
    ))
}
