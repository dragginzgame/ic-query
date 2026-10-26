use super::{
    fetch_topology_inventory_reports, summary::build_nns_topology_summary_report_with_source,
};
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION, NnsTopologyCheckReport,
        NnsTopologyCoverageReport, NnsTopologyHostError, NnsTopologyReadRequest, NnsTopologySource,
        NnsTopologyVersionsReport, check::topology_check_report_from_summary,
        coverage::topology_coverage_report_from_reports, enforce_mainnet_network,
        registry_versions::topology_registry_versions,
    },
};

pub async fn build_nns_topology_versions_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    build_nns_topology_versions_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_versions_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    enforce_mainnet_network(&request.network)?;

    let subnet_report = source.fetch_subnet_catalog_list_report(request).await?;
    let reports = fetch_topology_inventory_reports(request, source).await?;
    let registry_versions = topology_registry_versions(
        &subnet_report,
        &reports.node,
        &reports.node_provider,
        &reports.node_operator,
        &reports.data_center,
    );

    Ok(NnsTopologyVersionsReport {
        schema_version: NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        source_count: registry_versions.len(),
        registry_versions,
    })
}

pub async fn build_nns_topology_coverage_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    build_nns_topology_coverage_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_coverage_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    enforce_mainnet_network(&request.network)?;

    // Preserve the acquisition and failure order shared by topology reads.
    source.fetch_subnet_catalog_list_report(request).await?;
    let reports = fetch_topology_inventory_reports(request, source).await?;

    Ok(topology_coverage_report_from_reports(
        request.network.clone(),
        request.source_endpoint.clone(),
        &reports.node,
        &reports.node_provider,
        &reports.node_operator,
        &reports.data_center,
    ))
}

pub async fn build_nns_topology_check_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologyCheckReport, NnsTopologyHostError> {
    build_nns_topology_check_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_check_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCheckReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report_with_source(request, source).await?;

    Ok(topology_check_report_from_summary(summary))
}
