use crate::nns::{
    LiveNnsSource, NnsInventoryRefreshReport,
    data_center::NnsDataCenterHostError,
    node::NnsNodeHostError,
    node_operator::NnsNodeOperatorHostError,
    node_provider::NnsNodeProviderHostError,
    topology::report::{
        NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION, NnsTopologyHostError, NnsTopologyRefreshReport,
        NnsTopologyRefreshRequest, NnsTopologyRefreshRow, NnsTopologyRefreshSource,
        enforce_mainnet_network,
    },
};
use crate::subnet_catalog::SubnetCatalogRefreshReport;

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

    let components = vec![
        refresh_row_from_subnet_report(subnet_report),
        refresh_row_from_inventory_report(node_report),
        refresh_row_from_inventory_report(node_provider_report),
        refresh_row_from_inventory_report(node_operator_report),
        refresh_row_from_inventory_report(data_center_report),
    ];
    Ok(NnsTopologyRefreshReport {
        schema_version: NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION,
        network: request.read.network.clone(),
        source_endpoint: request.read.source_endpoint.clone(),
        dry_run: request.dry_run,
        component_count: components.len(),
        wrote_cache_count: components
            .iter()
            .filter(|component| component.wrote_cache)
            .count(),
        replaced_existing_cache_count: components
            .iter()
            .filter(|component| component.replaced_existing_cache)
            .count(),
        components,
    })
}

fn refresh_row_from_inventory_report(report: NnsInventoryRefreshReport) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: report.collection,
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.row_count,
    }
}

fn refresh_row_from_subnet_report(report: SubnetCatalogRefreshReport) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "subnet_catalog".to_string(),
        cache_path: report.catalog_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoints.join(","),
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_catalog,
        replaced_existing_cache: report.replaced_existing_catalog,
        item_count: report.subnet_count,
    }
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
