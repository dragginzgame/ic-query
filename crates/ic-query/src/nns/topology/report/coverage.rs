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
        node_count: reports.registry.nodes.len(),
        node_provider_count: reports.node_provider.node_providers.len(),
        node_operator_count: reports.registry.node_operators.len(),
        data_center_count: reports.registry.data_centers.len(),
        join_coverage: counts,
    }
}

pub(super) fn topology_join_coverage_counts(
    reports: &TopologyInventoryReports,
) -> NnsTopologyJoinCoverageCounts {
    let index = TopologyRelationIndex::from_reports(reports);

    let mut counts = NnsTopologyJoinCoverageCounts::default();
    for node in &reports.registry.nodes {
        let [provider, operator, data_center] = index.node_relations(node);
        counts.nodes_with_known_node_provider_count += usize::from(provider.known);
        counts.nodes_with_known_node_operator_count += usize::from(operator.known);
        counts.nodes_with_known_data_center_count += usize::from(data_center.known);
    }
    for operator in &reports.registry.node_operators {
        let [provider, data_center] = index.operator_relations(operator);
        counts.node_operators_with_known_node_provider_count += usize::from(provider.known);
        counts.node_operators_with_known_data_center_count += usize::from(data_center.known);
    }
    counts.nodes_with_unknown_node_provider_count = reports
        .registry
        .nodes
        .len()
        .saturating_sub(counts.nodes_with_known_node_provider_count);
    counts.nodes_with_unknown_node_operator_count = reports
        .registry
        .nodes
        .len()
        .saturating_sub(counts.nodes_with_known_node_operator_count);
    counts.nodes_with_unknown_data_center_count = reports
        .registry
        .nodes
        .len()
        .saturating_sub(counts.nodes_with_known_data_center_count);
    counts.node_operators_with_unknown_node_provider_count = reports
        .registry
        .node_operators
        .len()
        .saturating_sub(counts.node_operators_with_known_node_provider_count);
    counts.node_operators_with_unknown_data_center_count = reports
        .registry
        .node_operators
        .len()
        .saturating_sub(counts.node_operators_with_known_data_center_count);
    counts
}
