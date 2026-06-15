use super::super::{
    NnsTopologyCoverageReport, NnsTopologyCoverageRequest, NnsTopologyHealthReport,
    NnsTopologyHealthRequest, NnsTopologyHostError, NnsTopologyVersionsReport,
    NnsTopologyVersionsRequest, coverage::topology_coverage_report_from_summary,
    health::topology_health_report_from_summary, request::summary_request_from,
    versions::topology_versions_report_from_summary,
};
use super::summary::build_nns_topology_summary_report;

pub fn build_nns_topology_versions_report(
    request: &NnsTopologyVersionsRequest,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report(&summary_request_from(request))?;

    Ok(topology_versions_report_from_summary(summary))
}

pub fn build_nns_topology_coverage_report(
    request: &NnsTopologyCoverageRequest,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report(&summary_request_from(request))?;

    Ok(topology_coverage_report_from_summary(summary))
}

pub fn build_nns_topology_health_report(
    request: &NnsTopologyHealthRequest,
) -> Result<NnsTopologyHealthReport, NnsTopologyHostError> {
    let summary = build_nns_topology_summary_report(&summary_request_from(request))?;

    Ok(topology_health_report_from_summary(summary))
}
