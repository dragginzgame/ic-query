use super::{fixtures::*, *};

#[test]
fn topology_coverage_report_projects_summary_join_coverage() {
    let summary = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    let report = topology_coverage_report_from_summary(summary);

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.node_count, 3);
    assert_eq!(report.node_operator_count, 2);
    assert_eq!(report.node_provider_count, 1);
    assert_eq!(report.data_center_count, 1);
    assert_eq!(report.nodes_with_known_node_provider_count, 2);
    assert_eq!(report.nodes_with_unknown_node_provider_count, 1);
    assert_eq!(report.node_operators_with_known_data_center_count, 1);
    assert_eq!(report.node_operators_with_unknown_data_center_count, 1);
}

#[test]
fn topology_coverage_text_renders_join_coverage_table() {
    let summary = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );
    let report = topology_coverage_report_from_summary(summary);

    let text = nns_topology_coverage_report_text(&report);

    assert!(text.contains("FIELD"));
    assert!(text.contains("RELATION"));
    assert!(text.contains("\n\n"));
    assert!(text.contains("nodes -> node providers"));
    assert!(text.contains("node operators -> data centers"));
    assert!(text.contains("66.7%"));
}
