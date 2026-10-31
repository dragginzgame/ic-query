use super::fetch_full_topology_reports;
use crate::nns::NnsInventoryListRequest;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NnsTopologyHostError, NnsTopologySource, NnsTopologySummaryReport,
        summary::topology_summary_report_from_reports,
    },
};

pub async fn build_nns_topology_summary_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    build_nns_topology_summary_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn build_nns_topology_summary_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<NnsTopologySummaryReport, NnsTopologyHostError> {
    let (subnet_report, reports) = fetch_full_topology_reports(request, source).await?;

    Ok(topology_summary_report_from_reports(
        request.cache.network.clone(),
        request.source_endpoint.clone(),
        subnet_report,
        &reports,
    ))
}
