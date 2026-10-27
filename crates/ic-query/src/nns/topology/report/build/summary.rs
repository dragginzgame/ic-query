use super::fetch_topology_inventory_reports;
use crate::nns::{
    LiveNnsSource,
    topology::report::{
        NnsTopologyHostError, NnsTopologyReadRequest, NnsTopologySource, NnsTopologySummaryReport,
        enforce_mainnet_network, summary::topology_summary_report_from_reports,
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
    enforce_mainnet_network(request.network.as_str())?;

    let subnet_report = source.fetch_subnet_catalog_list_report(request).await?;
    let reports = fetch_topology_inventory_reports(request, source).await?;

    Ok(topology_summary_report_from_reports(
        request.network.clone(),
        request.source_endpoint.clone(),
        subnet_report,
        &reports,
    ))
}
