use crate::nns::{
    data_center::report::build_nns_data_center_list_report,
    node::report::build_nns_node_list_report,
    node_operator::report::build_nns_node_operator_list_report,
    node_provider::report::build_nns_node_provider_list_report,
    topology::report::{
        NnsTopologyCapacityReport, NnsTopologyCapacityRequest, NnsTopologyGapsReport,
        NnsTopologyGapsRequest, NnsTopologyHostError, NnsTopologyProvidersReport,
        NnsTopologyProvidersRequest, NnsTopologyRegionsReport, NnsTopologyRegionsRequest,
        capacity::topology_capacity_report_from_report,
        enforce_mainnet_network,
        gaps::topology_gaps_report_from_reports,
        providers::topology_providers_report_from_reports,
        regions::topology_regions_report_from_report,
        request::{
            TopologyRequestParts, data_center_list_request, node_list_request,
            node_operator_list_request, node_provider_list_request,
        },
    },
};

pub fn build_nns_topology_gaps_report(
    request: &NnsTopologyGapsRequest,
) -> Result<NnsTopologyGapsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let node_report = build_nns_node_list_report(&node_list_request(request))?;
    let node_provider_report =
        build_nns_node_provider_list_report(&node_provider_list_request(request))?;
    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;
    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

    Ok(topology_gaps_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ))
}

pub fn build_nns_topology_capacity_report(
    request: &NnsTopologyCapacityRequest,
) -> Result<NnsTopologyCapacityReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;

    Ok(topology_capacity_report_from_report(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_operator_report,
    ))
}

pub fn build_nns_topology_regions_report(
    request: &NnsTopologyRegionsRequest,
) -> Result<NnsTopologyRegionsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

    Ok(topology_regions_report_from_report(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        data_center_report,
    ))
}

pub fn build_nns_topology_providers_report(
    request: &NnsTopologyProvidersRequest,
) -> Result<NnsTopologyProvidersReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let node_report = build_nns_node_list_report(&node_list_request(request))?;
    let node_provider_report =
        build_nns_node_provider_list_report(&node_provider_list_request(request))?;
    let node_operator_report =
        build_nns_node_operator_list_report(&node_operator_list_request(request))?;
    let data_center_report = build_nns_data_center_list_report(&data_center_list_request(request))?;

    Ok(topology_providers_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ))
}
