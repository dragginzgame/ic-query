use crate::nns::NnsInventoryListRequest;
mod derived;
mod direct;
mod refresh;
mod summary;

use super::{
    NnsTopologyHostError, NnsTopologySource, TopologyInventoryReports, enforce_mainnet_network,
};
use crate::subnet_catalog::SubnetCatalogListReport;

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
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<TopologyInventoryReports, NnsTopologyHostError> {
    let mut progress = crate::progress::IgnoreQueryProgress;
    let snapshot =
        crate::nns::inventory::load_or_refresh_nns_inventory_report(request, source, &mut progress)
            .await?;
    let metadata = crate::nns::node_provider::load_or_refresh_metadata(
        request,
        source,
        &snapshot,
        &mut progress,
    )
    .await?;
    let node_provider =
        crate::nns::node_provider::node_provider_report_from_snapshot(&snapshot, metadata);
    Ok(TopologyInventoryReports {
        node: crate::nns::node::NnsNodeListReport {
            schema_version: 1,
            provenance: snapshot.provenance.clone(),
            nodes: snapshot.nodes,
        },
        node_operator: crate::nns::node_operator::NnsNodeOperatorListReport {
            schema_version: 1,
            provenance: snapshot.provenance.clone(),
            node_operators: snapshot.node_operators,
        },
        data_center: crate::nns::data_center::NnsDataCenterListReport {
            schema_version: 1,
            provenance: snapshot.provenance,
            data_centers: snapshot.data_centers,
        },
        node_provider,
    })
}

async fn fetch_full_topology_reports(
    request: &NnsInventoryListRequest,
    source: &dyn NnsTopologySource,
) -> Result<(SubnetCatalogListReport, TopologyInventoryReports), NnsTopologyHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let subnet_report = source.fetch_subnet_catalog_list_report(request).await?;
    let reports = fetch_topology_inventory_reports(request, source).await?;
    Ok((subnet_report, reports))
}
