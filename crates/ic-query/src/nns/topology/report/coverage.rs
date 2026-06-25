use super::{
    NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION, NnsTopologyCoverageReport,
    NnsTopologySummaryReport,
};

pub(super) fn topology_coverage_report_from_summary(
    summary: NnsTopologySummaryReport,
) -> NnsTopologyCoverageReport {
    NnsTopologyCoverageReport {
        schema_version: NNS_TOPOLOGY_COVERAGE_REPORT_SCHEMA_VERSION,
        network: summary.network,
        source_endpoint: summary.source_endpoint,
        node_count: summary.node_count,
        node_provider_count: summary.node_provider_count,
        node_operator_count: summary.node_operator_count,
        data_center_count: summary.data_center_count,
        nodes_with_known_node_provider_count: summary.nodes_with_known_node_provider_count,
        nodes_with_unknown_node_provider_count: summary.nodes_with_unknown_node_provider_count,
        nodes_with_known_node_operator_count: summary.nodes_with_known_node_operator_count,
        nodes_with_unknown_node_operator_count: summary.nodes_with_unknown_node_operator_count,
        nodes_with_known_data_center_count: summary.nodes_with_known_data_center_count,
        nodes_with_unknown_data_center_count: summary.nodes_with_unknown_data_center_count,
        node_operators_with_known_node_provider_count: summary
            .node_operators_with_known_node_provider_count,
        node_operators_with_unknown_node_provider_count: summary
            .node_operators_with_unknown_node_provider_count,
        node_operators_with_known_data_center_count: summary
            .node_operators_with_known_data_center_count,
        node_operators_with_unknown_data_center_count: summary
            .node_operators_with_unknown_data_center_count,
    }
}
