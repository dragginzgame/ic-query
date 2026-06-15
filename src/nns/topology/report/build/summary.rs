use super::super::{
    NnsTopologyHostError, NnsTopologySummaryReport, NnsTopologySummaryRequest,
    enforce_mainnet_network,
    request::{
        TopologyRequestParts, data_center_list_request, node_list_request,
        node_operator_list_request, node_provider_list_request, subnet_catalog_list_request,
    },
    summary::topology_summary_report_from_reports,
};
use crate::{
    nns::{
        data_center::report::build_nns_data_center_list_report,
        node::report::build_nns_node_list_report,
        node_operator::report::build_nns_node_operator_list_report,
        node_provider::report::build_nns_node_provider_list_report,
    },
    subnet_catalog::build_subnet_catalog_list_report,
};

pub fn build_nns_topology_summary_report(
    request: &NnsTopologySummaryRequest,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let subnet_report = build_subnet_catalog_list_report(&subnet_catalog_list_request(request))?;
    let node_report = build_nns_node_list_report(&node_list_request(request))?;
    let node_provider_report =
        build_nns_node_provider_list_report(&node_provider_list_request(request))?;
    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;
    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

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
