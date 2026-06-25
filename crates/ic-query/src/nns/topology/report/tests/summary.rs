use super::{fixtures::*, *};
use crate::test_support::temp_dir;

#[test]
fn topology_summary_counts_existing_reports() {
    let report = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    assert_eq!(report.schema_version, 3);
    assert_eq!(report.subnet_count, 2);
    assert_eq!(report.application_subnet_count, 1);
    assert_eq!(report.cloud_engine_subnet_count, 0);
    assert_eq!(report.system_subnet_count, 1);
    assert_eq!(report.routing_range_count, 3);
    assert_eq!(report.node_count, 3);
    assert_eq!(report.application_node_count, 2);
    assert_eq!(report.cloud_engine_node_count, 0);
    assert_eq!(report.system_node_count, 1);
    assert_eq!(report.node_provider_count, 1);
    assert_eq!(report.node_operator_count, 2);
    assert_eq!(report.data_center_count, 1);
    assert_eq!(report.nodes_with_known_node_provider_count, 2);
    assert_eq!(report.nodes_with_unknown_node_provider_count, 1);
    assert_eq!(report.nodes_with_known_node_operator_count, 2);
    assert_eq!(report.nodes_with_unknown_node_operator_count, 1);
    assert_eq!(report.nodes_with_known_data_center_count, 2);
    assert_eq!(report.nodes_with_unknown_data_center_count, 1);
    assert_eq!(report.node_operators_with_known_node_provider_count, 1);
    assert_eq!(report.node_operators_with_unknown_node_provider_count, 1);
    assert_eq!(report.node_operators_with_known_data_center_count, 1);
    assert_eq!(report.node_operators_with_unknown_data_center_count, 1);
    assert_eq!(report.registry_versions.len(), 5);
}

#[test]
fn topology_summary_text_renders_count_and_version_tables() {
    let report = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    let text = nns_topology_summary_report_text(&report);

    assert!(text.contains("topology: ic subnets 2 nodes 3"));
    assert!(text.contains("routing_ranges"));
    assert!(text.contains("KIND"));
    assert!(text.contains("nodes -> node providers"));
    assert!(text.contains("node operators -> data centers"));
    assert!(text.contains("COVERAGE"));
    assert!(text.contains("SOURCE"));
    assert!(text.contains("subnet_catalog"));
    assert!(text.contains("\n\n"));
}

#[test]
fn topology_summary_rejects_local_network_with_topology_hint() {
    let request = NnsTopologySummaryRequest {
        icp_root: temp_dir("ic-query-topology-local-network"),
        network: "local".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };

    let err = build_nns_topology_summary_report(&request).expect_err("local rejected");
    let message = err.to_string();

    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns topology summary"));
    assert!(message.contains("icq --network ic nns topology coverage"));
    assert!(message.contains("icq --network ic nns topology versions"));
    assert!(message.contains("icq --network ic nns topology health"));
    assert!(message.contains("icq --network ic nns topology gaps"));
    assert!(message.contains("icq --network ic nns topology capacity"));
    assert!(message.contains("icq --network ic nns topology regions"));
    assert!(message.contains("icq --network ic nns topology providers"));
    assert!(message.contains("icq --network ic nns topology refresh"));
}
