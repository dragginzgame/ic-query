use super::{fetch_full_topology_reports, fetch_topology_inventory_reports};
use crate::nns::NnsInventoryListRequest;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION, NnsTopologyCheckReport,
        NnsTopologyCoverageReport, NnsTopologyHostError, NnsTopologySource,
        NnsTopologyVersionsReport, check::topology_check_report_from_reports,
        coverage::topology_coverage_report_from_reports, enforce_mainnet_network,
        registry_versions::topology_registry_versions,
    },
};

pub async fn build_nns_topology_versions_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    build_nns_topology_versions_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_versions_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    let (subnet_report, reports) = fetch_full_topology_reports(request, source).await?;
    let registry_versions = topology_registry_versions(&subnet_report, &reports);

    Ok(NnsTopologyVersionsReport {
        schema_version: NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION,
        network: request.cache.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        source_count: registry_versions.len(),
        registry_versions,
    })
}

pub async fn build_nns_topology_coverage_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    build_nns_topology_coverage_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_coverage_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let reports = fetch_topology_inventory_reports(request, source).await?;

    Ok(topology_coverage_report_from_reports(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        &reports,
    ))
}

pub async fn build_nns_topology_check_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologyCheckReport, NnsTopologyHostError> {
    build_nns_topology_check_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_check_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCheckReport, NnsTopologyHostError> {
    let (subnet_report, reports) = fetch_full_topology_reports(request, source).await?;
    Ok(topology_check_report_from_reports(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        subnet_report,
        &reports,
    ))
}
