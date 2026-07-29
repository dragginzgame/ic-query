use super::summary::build_nns_topology_summary_report_with_source;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NnsTopologyCoverageReport, NnsTopologyHealthReport, NnsTopologyHostError,
        NnsTopologyReadRequest, NnsTopologySource, NnsTopologyVersionsReport,
        coverage::topology_coverage_report_from_summary,
        health::topology_health_report_from_summary, request::summary_request_from,
        versions::topology_versions_report_from_summary,
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
    let summary =
        build_nns_topology_summary_report_with_source(&summary_request_from(request), source)?;

    Ok(topology_versions_report_from_summary(summary))
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
    let summary =
        build_nns_topology_summary_report_with_source(&summary_request_from(request), source)?;

    Ok(topology_coverage_report_from_summary(summary))
}

pub fn build_nns_topology_health_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologyHealthReport, NnsTopologyHostError> {
    build_nns_topology_health_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_topology_health_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyHealthReport, NnsTopologyHostError> {
    let summary =
        build_nns_topology_summary_report_with_source(&summary_request_from(request), source)?;

    Ok(topology_health_report_from_summary(summary))
}
