use super::{
    fetch_topology_inventory_reports, summary::build_nns_topology_summary_report_with_source,
};
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION, NnsTopologyCheckReport,
        NnsTopologyCoverageReport, NnsTopologyHostError, NnsTopologyReadRequest, NnsTopologySource,
        NnsTopologyVersionsReport, check::topology_check_report_from_summary,
        coverage::topology_coverage_report_from_summary, enforce_mainnet_network,
        registry_versions::topology_registry_versions, source::topology_source_request_from,
    },
};

pub fn build_nns_topology_versions_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    build_nns_topology_versions_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_topology_versions_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    enforce_mainnet_network(&request.network)?;

    let source_request = topology_source_request_from(request);
    let subnet_report = source.fetch_subnet_catalog_list_report(&source_request)?;
    let reports = fetch_topology_inventory_reports(&source_request, source)?;
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

pub fn build_nns_topology_coverage_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    build_nns_topology_coverage_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_topology_coverage_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report_with_source(request, source)?;

    Ok(topology_coverage_report_from_summary(summary))
}

pub fn build_nns_topology_check_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologyCheckReport, NnsTopologyHostError> {
    build_nns_topology_check_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_topology_check_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCheckReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report_with_source(request, source)?;

    Ok(topology_check_report_from_summary(summary))
}
