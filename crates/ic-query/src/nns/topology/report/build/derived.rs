use super::summary::build_nns_topology_summary_report_with_source;
use crate::nns::topology::report::{
    LiveNnsTopologySource, NnsTopologyCoverageReport, NnsTopologyCoverageRequest,
    NnsTopologyHealthReport, NnsTopologyHealthRequest, NnsTopologyHostError, NnsTopologySource,
    NnsTopologyVersionsReport, NnsTopologyVersionsRequest,
    coverage::topology_coverage_report_from_summary, health::topology_health_report_from_summary,
    request::summary_request_from, versions::topology_versions_report_from_summary,
};

pub fn build_nns_topology_versions_report(
    request: &NnsTopologyVersionsRequest,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    build_nns_topology_versions_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_versions_report_with_source(
    request: &NnsTopologyVersionsRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyVersionsReport, NnsTopologyHostError> {
    let summary =
        build_nns_topology_summary_report_with_source(&summary_request_from(request), source)?;

    Ok(topology_versions_report_from_summary(summary))
}

pub fn build_nns_topology_coverage_report(
    request: &NnsTopologyCoverageRequest,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    build_nns_topology_coverage_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_coverage_report_with_source(
    request: &NnsTopologyCoverageRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyCoverageReport, NnsTopologyHostError> {
    let summary =
        build_nns_topology_summary_report_with_source(&summary_request_from(request), source)?;

    Ok(topology_coverage_report_from_summary(summary))
}

pub fn build_nns_topology_health_report(
    request: &NnsTopologyHealthRequest,
) -> Result<NnsTopologyHealthReport, NnsTopologyHostError> {
    build_nns_topology_health_report_with_source(request, &LiveNnsTopologySource)
}

pub fn build_nns_topology_health_report_with_source(
    request: &NnsTopologyHealthRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologyHealthReport, NnsTopologyHostError> {
    let summary =
        build_nns_topology_summary_report_with_source(&summary_request_from(request), source)?;

    Ok(topology_health_report_from_summary(summary))
}
