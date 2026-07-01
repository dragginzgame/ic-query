use crate::nns::topology::report::{
    LiveNnsTopologySource, NnsTopologyCapacityReport, NnsTopologyCapacityRequest,
    NnsTopologyGapsReport, NnsTopologyGapsRequest, NnsTopologyHostError,
    NnsTopologyProvidersReport, NnsTopologyProvidersRequest, NnsTopologyRegionsReport,
    NnsTopologyRegionsRequest, NnsTopologySource, capacity::topology_capacity_report_from_report,
    enforce_mainnet_network, gaps::topology_gaps_report_from_reports,
    providers::topology_providers_report_from_reports,
    regions::topology_regions_report_from_report, request::TopologyRequestParts,
    source::topology_source_request_from,
};

pub fn build_nns_topology_gaps_report(
    request: &NnsTopologyGapsRequest,
) -> Result<NnsTopologyGapsReport, NnsTopologyHostError> {
    build_nns_topology_gaps_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_gaps_report_with_source(
    request: &NnsTopologyGapsRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyGapsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let source_request = topology_source_request_from(request);
    let node_report = source.fetch_node_list_report(&source_request)?;
    let node_provider_report = source.fetch_node_provider_list_report(&source_request)?;
    let node_operator_report = source.fetch_node_operator_list_report(&source_request)?;
    let data_center_report = source.fetch_data_center_list_report(&source_request)?;

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
    build_nns_topology_capacity_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_capacity_report_with_source(
    request: &NnsTopologyCapacityRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCapacityReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let source_request = topology_source_request_from(request);
    let node_operator_report = source.fetch_node_operator_list_report(&source_request)?;

    Ok(topology_capacity_report_from_report(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_operator_report,
    ))
}

pub fn build_nns_topology_regions_report(
    request: &NnsTopologyRegionsRequest,
) -> Result<NnsTopologyRegionsReport, NnsTopologyHostError> {
    build_nns_topology_regions_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_regions_report_with_source(
    request: &NnsTopologyRegionsRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyRegionsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let source_request = topology_source_request_from(request);
    let data_center_report = source.fetch_data_center_list_report(&source_request)?;

    Ok(topology_regions_report_from_report(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        data_center_report,
    ))
}

pub fn build_nns_topology_providers_report(
    request: &NnsTopologyProvidersRequest,
) -> Result<NnsTopologyProvidersReport, NnsTopologyHostError> {
    build_nns_topology_providers_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_providers_report_with_source(
    request: &NnsTopologyProvidersRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyProvidersReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.network())?;

    let source_request = topology_source_request_from(request);
    let node_report = source.fetch_node_list_report(&source_request)?;
    let node_provider_report = source.fetch_node_provider_list_report(&source_request)?;
    let node_operator_report = source.fetch_node_operator_list_report(&source_request)?;
    let data_center_report = source.fetch_data_center_list_report(&source_request)?;

    Ok(topology_providers_report_from_reports(
        request.network().to_string(),
        request.source_endpoint().to_string(),
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ))
}
