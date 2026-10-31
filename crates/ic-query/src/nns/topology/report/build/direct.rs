use super::fetch_topology_inventory_reports;
use crate::nns::NnsInventoryListRequest;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NnsTopologyCapacityReport, NnsTopologyGapsReport, NnsTopologyHostError,
        NnsTopologyProvidersReport, NnsTopologyRegionsReport, NnsTopologySource,
        capacity::topology_capacity_report_from_report, enforce_mainnet_network,
        gaps::topology_gaps_report_from_reports, providers::topology_providers_report_from_reports,
        regions::topology_regions_report_from_report,
    },
};

pub async fn build_nns_topology_gaps_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologyGapsReport, NnsTopologyHostError> {
    build_nns_topology_gaps_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_gaps_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyGapsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.cache.network.as_str())?;

    let reports = fetch_topology_inventory_reports(request, source).await?;

    Ok(topology_gaps_report_from_reports(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        &reports,
    ))
}

pub async fn build_nns_topology_capacity_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologyCapacityReport, NnsTopologyHostError> {
    build_nns_topology_capacity_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_capacity_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCapacityReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.cache.network.as_str())?;

    let node_operator_report = source.fetch_node_operator_list_report(request).await?;

    Ok(topology_capacity_report_from_report(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        node_operator_report,
    ))
}

pub async fn build_nns_topology_regions_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologyRegionsReport, NnsTopologyHostError> {
    build_nns_topology_regions_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_regions_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyRegionsReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.cache.network.as_str())?;

    let data_center_report = source.fetch_data_center_list_report(request).await?;

    Ok(topology_regions_report_from_report(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        data_center_report,
    ))
}

pub async fn build_nns_topology_providers_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologyProvidersReport, NnsTopologyHostError> {
    build_nns_topology_providers_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_providers_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyProvidersReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.cache.network.as_str())?;

    let reports = fetch_topology_inventory_reports(request, source).await?;

    Ok(topology_providers_report_from_reports(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        &reports,
    ))
}
