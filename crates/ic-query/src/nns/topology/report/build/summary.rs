use super::fetch_full_topology_reports;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NnsTopologyHostError, NnsTopologyReadRequest, NnsTopologySource, NnsTopologySummaryReport,
        summary::topology_summary_report_from_reports,
    },
};

pub async fn build_nns_topology_summary_report(
    request: &NnsTopologyReadRequest,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    build_nns_topology_summary_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_summary_report_with_source(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    let (subnet_report, reports) = fetch_full_topology_reports(request, source).await?;

    Ok(topology_summary_report_from_reports(
        request.network.clone(),
        request.source_endpoint.clone(),
        subnet_report,
        &reports,
    ))
}
