use super::{
    NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION, NnsTopologyCoverageReport,
    NnsTopologyJoinCoverageCounts, TopologyInventoryReports, relations::TopologyRelationIndex,
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
        join_coverage: counts,
    }
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
        let [provider, operator, data_center] = index.node_relations(node);
        counts.nodes_with_known_node_provider_count += usize::from(provider.known);
        counts.nodes_with_known_node_operator_count += usize::from(operator.known);
        counts.nodes_with_known_data_center_count += usize::from(data_center.known);
    }
    for operator in &reports.node_operator.node_operators {
        let [provider, data_center] = index.operator_relations(operator);
        counts.node_operators_with_known_node_provider_count += usize::from(provider.known);
        counts.node_operators_with_known_data_center_count += usize::from(data_center.known);
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
