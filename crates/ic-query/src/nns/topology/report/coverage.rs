use super::{NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION, NnsTopologyCoverageReport};
use crate::nns::{
    data_center::NnsDataCenterListReport, node::NnsNodeListReport,
    node_operator::NnsNodeOperatorListReport, node_provider::NnsNodeProviderListReport,
    topology::report::relations::TopologyRelationIndex,
};

pub(super) fn topology_coverage_report_from_reports(
    network: String,
    source_endpoint: String,
    node_report: &NnsNodeListReport,
    node_provider_report: &NnsNodeProviderListReport,
    node_operator_report: &NnsNodeOperatorListReport,
    data_center_report: &NnsDataCenterListReport,
) -> NnsTopologyCoverageReport {
    let counts = topology_join_coverage_counts(
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    );
    NnsTopologyCoverageReport {
        schema_version: NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        node_count: node_report.node_count,
        node_provider_count: node_provider_report.node_provider_count,
        node_operator_count: node_operator_report.node_operator_count,
        data_center_count: data_center_report.data_center_count,
        nodes_with_known_node_provider_count: counts.nodes_with_known_node_provider_count,
        nodes_with_unknown_node_provider_count: node_report
            .node_count
            .saturating_sub(counts.nodes_with_known_node_provider_count),
        nodes_with_known_node_operator_count: counts.nodes_with_known_node_operator_count,
        nodes_with_unknown_node_operator_count: node_report
            .node_count
            .saturating_sub(counts.nodes_with_known_node_operator_count),
        nodes_with_known_data_center_count: counts.nodes_with_known_data_center_count,
        nodes_with_unknown_data_center_count: node_report
            .node_count
            .saturating_sub(counts.nodes_with_known_data_center_count),
        node_operators_with_known_node_provider_count: counts
            .node_operators_with_known_node_provider_count,
        node_operators_with_unknown_node_provider_count: node_operator_report
            .node_operator_count
            .saturating_sub(counts.node_operators_with_known_node_provider_count),
        node_operators_with_known_data_center_count: counts
            .node_operators_with_known_data_center_count,
        node_operators_with_unknown_data_center_count: node_operator_report
            .node_operator_count
            .saturating_sub(counts.node_operators_with_known_data_center_count),
    }
}

///
/// NnsTopologyJoinCoverageCounts
///
/// Internal known-relation counters shared by coverage and summary reports.
///

#[derive(Default)]
#[expect(
    clippy::struct_field_names,
    reason = "fields mirror the public topology coverage count names"
)]
struct NnsTopologyJoinCoverageCounts {
    nodes_with_known_node_provider_count: usize,
    nodes_with_known_node_operator_count: usize,
    nodes_with_known_data_center_count: usize,
    node_operators_with_known_node_provider_count: usize,
    node_operators_with_known_data_center_count: usize,
}

fn topology_join_coverage_counts(
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
