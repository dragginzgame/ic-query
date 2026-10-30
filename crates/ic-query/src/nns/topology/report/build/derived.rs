use super::{fetch_full_topology_reports, summary::build_nns_topology_summary_report_with_source};
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NNS_TOPOLOGY_VERSIONS_REPORT_SCHEMA_VERSION, NnsTopologyCheckReport,
        NnsTopologyCoverageReport, NnsTopologyHostError, NnsTopologyReadRequest, NnsTopologySource,
        NnsTopologyVersionsReport, check::topology_check_report_from_summary,
        coverage::topology_coverage_report_from_reports,
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
    let (subnet_report, reports) = fetch_full_topology_reports(request, source).await?;
    let registry_versions = topology_registry_versions(&subnet_report, &reports);

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
    let (_, reports) = fetch_full_topology_reports(request, source).await?;

    Ok(topology_coverage_report_from_reports(
        request.network.clone(),
        request.source_endpoint.clone(),
        &reports,
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
