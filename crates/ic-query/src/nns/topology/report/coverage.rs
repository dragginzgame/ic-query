use super::{
    NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION, NnsTopologyCoverageReport,
    TopologyInventoryReports, relations::TopologyRelationIndex,
};

pub(super) fn topology_coverage_report_from_reports(
    network: String,
    source_endpoint: String,
    reports: &TopologyInventoryReports,
) -> NnsTopologyCoverageReport {
    let counts = topology_join_coverage_counts(reports);
    NnsTopologyCoverageReport {
        schema_version: NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        node_count: reports.node.node_count,
        node_provider_count: reports.node_provider.node_provider_count,
        node_operator_count: reports.node_operator.node_operator_count,
        data_center_count: reports.data_center.data_center_count,
        nodes_with_known_node_provider_count: counts.nodes_with_known_node_provider_count,
        nodes_with_unknown_node_provider_count: counts.nodes_with_unknown_node_provider_count,
        nodes_with_known_node_operator_count: counts.nodes_with_known_node_operator_count,
        nodes_with_unknown_node_operator_count: counts.nodes_with_unknown_node_operator_count,
        nodes_with_known_data_center_count: counts.nodes_with_known_data_center_count,
        nodes_with_unknown_data_center_count: counts.nodes_with_unknown_data_center_count,
        node_operators_with_known_node_provider_count: counts
            .node_operators_with_known_node_provider_count,
        node_operators_with_unknown_node_provider_count: counts
            .node_operators_with_unknown_node_provider_count,
        node_operators_with_known_data_center_count: counts
            .node_operators_with_known_data_center_count,
        node_operators_with_unknown_data_center_count: counts
            .node_operators_with_unknown_data_center_count,
    }
}

///
/// NnsTopologyJoinCoverageCounts
///
/// Internal relation-coverage counts shared by coverage and summary reports.
///

#[derive(Default)]
#[expect(
    clippy::struct_field_names,
    reason = "fields mirror the public topology coverage count names"
)]
pub(super) struct NnsTopologyJoinCoverageCounts {
    pub(super) nodes_with_known_node_provider_count: usize,
    pub(super) nodes_with_unknown_node_provider_count: usize,
    pub(super) nodes_with_known_node_operator_count: usize,
    pub(super) nodes_with_unknown_node_operator_count: usize,
    pub(super) nodes_with_known_data_center_count: usize,
    pub(super) nodes_with_unknown_data_center_count: usize,
    pub(super) node_operators_with_known_node_provider_count: usize,
    pub(super) node_operators_with_unknown_node_provider_count: usize,
    pub(super) node_operators_with_known_data_center_count: usize,
    pub(super) node_operators_with_unknown_data_center_count: usize,
}

pub(super) fn topology_join_coverage_counts(
    reports: &TopologyInventoryReports,
) -> NnsTopologyJoinCoverageCounts {
    let index = TopologyRelationIndex::from_reports(
        &reports.node_provider,
        &reports.node_operator,
        &reports.data_center,
    );

    let mut counts = NnsTopologyJoinCoverageCounts::default();
    for node in &reports.node.nodes {
        counts.nodes_with_known_node_provider_count +=
            usize::from(index.has_node_provider(&node.node_provider_principal));
        counts.nodes_with_known_node_operator_count +=
            usize::from(index.has_node_operator(&node.node_operator_principal));
        counts.nodes_with_known_data_center_count +=
            usize::from(index.has_data_center(&node.data_center_id));
    }
    for operator in &reports.node_operator.node_operators {
        counts.node_operators_with_known_node_provider_count +=
            usize::from(index.has_node_provider(&operator.node_provider_principal));
        counts.node_operators_with_known_data_center_count +=
            usize::from(index.has_data_center(&operator.data_center_id));
    }
    counts.nodes_with_unknown_node_provider_count = reports
        .node
        .node_count
        .saturating_sub(counts.nodes_with_known_node_provider_count);
    counts.nodes_with_unknown_node_operator_count = reports
        .node
        .node_count
        .saturating_sub(counts.nodes_with_known_node_operator_count);
    counts.nodes_with_unknown_data_center_count = reports
        .node
        .node_count
        .saturating_sub(counts.nodes_with_known_data_center_count);
    counts.node_operators_with_unknown_node_provider_count = reports
        .node_operator
        .node_operator_count
        .saturating_sub(counts.node_operators_with_known_node_provider_count);
    counts.node_operators_with_unknown_data_center_count = reports
        .node_operator
        .node_operator_count
        .saturating_sub(counts.node_operators_with_known_data_center_count);
    counts
}
