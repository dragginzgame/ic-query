use super::fetch_topology_inventory_reports;
use crate::nns::NnsInventoryListRequest;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NnsTopologyCapacityReport, NnsTopologyGapsReport, NnsTopologyHostError,
        NnsTopologyProvidersReport, NnsTopologyRegionsReport, NnsTopologySource,
        capacity::topology_capacity_report_from_rows, enforce_mainnet_network,
        gaps::topology_gaps_report_from_reports, providers::topology_providers_report_from_reports,
        regions::topology_regions_report_from_rows,
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

    let snapshot = crate::nns::inventory::load_or_refresh_nns_inventory_report(
        request,
        source,
        &mut crate::progress::IgnoreQueryProgress,
    )
    .await?;

    Ok(topology_capacity_report_from_rows(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        snapshot.node_operators,
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

    let snapshot = crate::nns::inventory::load_or_refresh_nns_inventory_report(
        request,
        source,
        &mut crate::progress::IgnoreQueryProgress,
    )
    .await?;

    Ok(topology_regions_report_from_rows(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        &snapshot.data_centers,
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
