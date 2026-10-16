use crate::nns::{
    LiveNnsSource, NnsInventoryRefreshReport,
    data_center::NnsDataCenterHostError,
    node::NnsNodeHostError,
    node_operator::NnsNodeOperatorHostError,
    node_provider::NnsNodeProviderHostError,
    topology::report::{
        NnsTopologyHostError, NnsTopologyRefreshReport, NnsTopologyRefreshRequest,
        NnsTopologyRefreshSource, enforce_mainnet_network,
        refresh::{NnsTopologyRefreshComponentReports, topology_refresh_report_from_reports},
    },
};

pub async fn refresh_nns_topology_report(
    request: &NnsTopologyRefreshRequest,
) -> Result<NnsTopologyRefreshReport, NnsTopologyHostError> {
    refresh_nns_topology_report_with_source(request, &LiveNnsSource::default()).await
}

pub async fn refresh_nns_topology_report_with_source(
    request: &NnsTopologyRefreshRequest,
    source: &dyn NnsTopologyRefreshSource,
) -> Result<NnsTopologyRefreshReport, NnsTopologyHostError> {
    enforce_mainnet_network(request.read.network.as_str())?;

    let subnet_report = source.refresh_subnet_catalog_report(request).await?;
    let node_report = source.refresh_node_report(request).await?;
    validate_inventory_refresh_collection(&node_report, "nodes")
        .map_err(|reason| NnsNodeHostError::InvalidSourceData { reason })?;
    let node_provider_report = source.refresh_node_provider_report(request).await?;
    validate_inventory_refresh_collection(&node_provider_report, "node_providers")
        .map_err(|reason| NnsNodeProviderHostError::InvalidSourceData { reason })?;
    let node_operator_report = source.refresh_node_operator_report(request).await?;
    validate_inventory_refresh_collection(&node_operator_report, "node_operators")
        .map_err(|reason| NnsNodeOperatorHostError::InvalidSourceData { reason })?;
    let data_center_report = source.refresh_data_center_report(request).await?;
    validate_inventory_refresh_collection(&data_center_report, "data_centers")
        .map_err(|reason| NnsDataCenterHostError::InvalidSourceData { reason })?;

    Ok(topology_refresh_report_from_reports(
        request.read.network.clone(),
        request.read.source_endpoint.clone(),
        request.dry_run,
        NnsTopologyRefreshComponentReports {
            subnet: subnet_report,
            node: node_report,
            node_provider: node_provider_report,
            node_operator: node_operator_report,
            data_center: data_center_report,
        },
    ))
}

fn validate_inventory_refresh_collection(
    report: &NnsInventoryRefreshReport,
    expected: &str,
) -> Result<(), String> {
    if report.collection != expected {
        return Err(format!(
            "refresh collection is {:?}, expected {expected}",
            report.collection
        ));
    }
    Ok(())
}
