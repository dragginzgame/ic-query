use super::{fixtures::*, *};

#[test]
fn topology_versions_report_projects_summary_registry_versions() {
    let summary = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    let report = topology_versions_report_from_summary(summary);

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.source_count, 5);
    assert_eq!(report.registry_versions[0].source, "subnet_catalog");
    assert_eq!(report.registry_versions[1].source, "nodes");
}

#[test]
fn topology_versions_text_renders_registry_version_table() {
    let summary = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );
    let report = topology_versions_report_from_summary(summary);

    let text = nns_topology_versions_report_text(&report);

    assert!(text.contains("SOURCE"));
    assert!(text.contains("VERSION"));
    assert!(text.contains("subnet_catalog"));
    assert!(text.contains("node_operators"));
    assert!(text.contains("data_centers"));
}
