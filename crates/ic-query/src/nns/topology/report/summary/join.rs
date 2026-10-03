//! Module: nns::topology::report::summary::join
//!
//! Responsibility: count known and unknown NNS topology relation joins.
//! Does not own: component reports, registry versions, or text rendering.
//! Boundary: summarizes relation coverage for topology summary reports.

use crate::nns::{
    data_center::NnsDataCenterListReport, node::NnsNodeListReport,
    node_operator::NnsNodeOperatorListReport, node_provider::NnsNodeProviderListReport,
    topology::report::relations::TopologyRelationIndex,
};

///
/// NnsTopologyJoinCoverageCounts
///
/// Internal known-relation counters used by topology summary assembly.
///

#[derive(Default)]
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

    let mut counts = NnsTopologyJoinCoverageCounts::default();
    for node in &node_report.nodes {
        counts.nodes_with_known_node_provider_count +=
            usize::from(index.has_node_provider(&node.node_provider_principal));
        counts.nodes_with_known_node_operator_count +=
            usize::from(index.has_node_operator(&node.node_operator_principal));
        counts.nodes_with_known_data_center_count +=
            usize::from(index.has_data_center(&node.data_center_id));
    }
    for operator in &node_operator_report.node_operators {
        counts.node_operators_with_known_node_provider_count +=
            usize::from(index.has_node_provider(&operator.node_provider_principal));
        counts.node_operators_with_known_data_center_count +=
            usize::from(index.has_data_center(&operator.data_center_id));
    }
    counts
}
