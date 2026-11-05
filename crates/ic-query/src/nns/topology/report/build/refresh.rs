use crate::nns::{
    LiveNnsSource, NnsInventoryRefreshReport,
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
    enforce_mainnet_network(request.read.cache.network.as_str())?;

    let subnet_report = source.refresh_subnet_catalog_report(request).await?;
    let inventory = crate::nns::node_provider::refresh_nns_node_provider_report_with_source(
        &super::super::request::inventory_refresh_request(request),
        source,
    )
    .await?;
    let components = vec![
        refresh_row_from_subnet_report(subnet_report),
        refresh_row_from_inventory_report(inventory.inventory),
        refresh_row_from_inventory_report(inventory.metadata),
    ];
    Ok(NnsTopologyRefreshReport {
        schema_version: NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION,
        network: request.read.cache.network.clone(),
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
    let (registry_version, fetched_at, source_endpoint, fetched_by) = match report.governance_source
    {
        Some(source) => (None, source.fetched_at, source.endpoint, source.fetched_by),
        None => (
            Some(report.provenance.registry_version),
            report.provenance.fetched_at,
            report.provenance.source_endpoint,
            report.provenance.fetched_by,
        ),
    };
    NnsTopologyRefreshRow {
        source: report.collection,
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version,
        fetched_at,
        source_endpoint,
        fetched_by,
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
        registry_version: Some(report.metadata.registry_version),
        fetched_at: report.metadata.fetched_at,
        source_endpoint: report.metadata.source_endpoints.join(","),
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_catalog,
        replaced_existing_cache: report.replaced_existing_catalog,
        item_count: report.subnet_count,
    }
}
