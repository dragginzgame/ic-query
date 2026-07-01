//! Module: nns::topology::report::summary::join
//!
//! Responsibility: count known and unknown NNS topology relation joins.
//! Does not own: component reports, registry versions, or text rendering.
//! Boundary: summarizes relation coverage for topology summary reports.

use crate::nns::{
    data_center::report::NnsDataCenterListReport, node::report::NnsNodeListReport,
    node_operator::report::NnsNodeOperatorListReport,
    node_provider::report::NnsNodeProviderListReport,
    topology::report::relations::TopologyRelationIndex,
};

///
/// NnsTopologyJoinCoverageCounts
///
/// Internal known-relation counters used by topology summary assembly.
///
#[expect(
    clippy::struct_field_names,
    reason = "fields mirror the public topology summary count names"
)]
pub(super) struct NnsTopologyJoinCoverageCounts {
    pub(super) nodes_with_known_node_provider_count: usize,
    pub(super) nodes_with_known_node_operator_count: usize,
    pub(super) nodes_with_known_data_center_count: usize,
    pub(super) node_operators_with_known_node_provider_count: usize,
    pub(super) node_operators_with_known_data_center_count: usize,
}

pub(super) fn topology_summary_join_coverage_counts(
    node_report: &NnsNodeListReport,
    node_provider_report: &NnsNodeProviderListReport,
    node_operator_report: &NnsNodeOperatorListReport,
    data_center_report: &NnsDataCenterListReport,
) -> NnsTopologyJoinCoverageCounts {
    let index = TopologyRelationIndex::from_reports(
        node_provider_report,
        node_operator_report,
        data_center_report,
    );

    NnsTopologyJoinCoverageCounts {
        nodes_with_known_node_provider_count: index
            .nodes_with_known_node_provider_count(node_report),
        nodes_with_known_node_operator_count: index
            .nodes_with_known_node_operator_count(node_report),
        nodes_with_known_data_center_count: index.nodes_with_known_data_center_count(node_report),
        node_operators_with_known_node_provider_count: index
            .node_operators_with_known_node_provider_count(node_operator_report),
        node_operators_with_known_data_center_count: index
            .node_operators_with_known_data_center_count(node_operator_report),
    }
}
