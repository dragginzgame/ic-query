mod derived;
mod direct;
mod refresh;
mod summary;

use super::{NnsTopologyHostError, NnsTopologySource, NnsTopologySourceRequest};
use crate::nns::{
    data_center::NnsDataCenterListReport, node::NnsNodeListReport,
    node_operator::NnsNodeOperatorListReport, node_provider::NnsNodeProviderListReport,
};

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

///
/// TopologyInventoryReports
///
/// Complete component inventories collected in diagnostic read order.
///

struct TopologyInventoryReports {
    node: NnsNodeListReport,
    node_provider: NnsNodeProviderListReport,
    node_operator: NnsNodeOperatorListReport,
    data_center: NnsDataCenterListReport,
}

fn fetch_topology_inventory_reports(
    request: &NnsTopologySourceRequest,
    source: &dyn NnsTopologySource,
) -> Result<TopologyInventoryReports, NnsTopologyHostError> {
    Ok(TopologyInventoryReports {
        node: source.fetch_node_list_report(request)?,
        node_provider: source.fetch_node_provider_list_report(request)?,
        node_operator: source.fetch_node_operator_list_report(request)?,
        data_center: source.fetch_data_center_list_report(request)?,
    })
}
