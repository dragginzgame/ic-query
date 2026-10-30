mod derived;
mod direct;
mod refresh;
mod summary;

use super::{
    NnsTopologyHostError, NnsTopologySource, TopologyInventoryReports, enforce_mainnet_network,
};
use crate::{nns::topology::NnsTopologyReadRequest, subnet_catalog::SubnetCatalogListReport};

pub use derived::{
    build_nns_topology_check_report, build_nns_topology_check_report_with_source,
    build_nns_topology_coverage_report, build_nns_topology_coverage_report_with_source,
    build_nns_topology_versions_report, build_nns_topology_versions_report_with_source,
};
pub use direct::{
    build_nns_topology_capacity_report, build_nns_topology_capacity_report_with_source,
    build_nns_topology_gaps_report, build_nns_topology_gaps_report_with_source,
    build_nns_topology_providers_report, build_nns_topology_providers_report_with_source,
    build_nns_topology_regions_report, build_nns_topology_regions_report_with_source,
};
pub use refresh::{refresh_nns_topology_report, refresh_nns_topology_report_with_source};
pub use summary::{
    build_nns_topology_summary_report, build_nns_topology_summary_report_with_source,
};

async fn fetch_topology_inventory_reports(
    request: &crate::nns::topology::NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<TopologyInventoryReports, NnsTopologyHostError> {
    Ok(TopologyInventoryReports {
        node: source.fetch_node_list_report(request).await?,
        node_provider: source.fetch_node_provider_list_report(request).await?,
        node_operator: source.fetch_node_operator_list_report(request).await?,
        data_center: source.fetch_data_center_list_report(request).await?,
    })
}

async fn fetch_full_topology_reports(
    request: &NnsTopologyReadRequest,
    source: &dyn NnsTopologySource,
) -> Result<(SubnetCatalogListReport, TopologyInventoryReports), NnsTopologyHostError> {
    enforce_mainnet_network(&request.network)?;
    let subnet_report = source.fetch_subnet_catalog_list_report(request).await?;
    let reports = fetch_topology_inventory_reports(request, source).await?;
    Ok((subnet_report, reports))
}
