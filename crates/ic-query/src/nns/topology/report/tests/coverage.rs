use super::fixtures::*;
use super::*;

#[test]
fn topology_coverage_report_counts_inventory_relations() {
    let report = topology_coverage_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        &TopologyInventoryReports {
            node: node_report_fixture(),
            node_provider: node_provider_report_fixture(),
            node_operator: node_operator_report_fixture(),
            data_center: data_center_report_fixture(),
        },
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.node_count, 3);
    assert_eq!(report.node_operator_count, 2);
    assert_eq!(report.node_provider_count, 1);
    assert_eq!(report.data_center_count, 1);
    assert_eq!(report.join_coverage.nodes_with_known_node_provider_count, 2);
    assert_eq!(
        report.join_coverage.nodes_with_unknown_node_provider_count,
        1
    );
    assert_eq!(
        report
            .join_coverage
            .node_operators_with_known_data_center_count,
        1
    );
    assert_eq!(
        report
            .join_coverage
            .node_operators_with_unknown_data_center_count,
        1
    );
}
