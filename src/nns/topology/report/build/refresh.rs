use super::super::{
    NnsTopologyHostError, NnsTopologyRefreshReport, NnsTopologyRefreshRequest,
    enforce_mainnet_network,
    refresh::{NnsTopologyRefreshComponentReports, topology_refresh_report_from_reports},
    request::{
        TopologyRefreshParts, TopologyRequestParts, data_center_refresh_request,
        node_operator_refresh_request, node_provider_refresh_request, node_refresh_request,
        subnet_catalog_refresh_request,
    },
};
use crate::{
    nns::{
        data_center::report::refresh_nns_data_center_report, node::report::refresh_nns_node_report,
        node_operator::report::refresh_nns_node_operator_report,
        node_provider::report::refresh_nns_node_provider_report,
    },
    subnet_catalog::refresh_subnet_catalog,
};

pub fn refresh_nns_topology_report(
    request: &NnsTopologyRefreshRequest,
) -> Result<NnsTopologyRefreshReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let subnet_report = refresh_subnet_catalog(&subnet_catalog_refresh_request(request))?;
    let node_report = refresh_nns_node_report(&node_refresh_request(request))?;
    let node_provider_report =
        refresh_nns_node_provider_report(&node_provider_refresh_request(request))?;
    let node_operator_report =
        refresh_nns_node_operator_report(&node_operator_refresh_request(request))?;
    let data_center_report = refresh_nns_data_center_report(&data_center_refresh_request(request))?;

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
