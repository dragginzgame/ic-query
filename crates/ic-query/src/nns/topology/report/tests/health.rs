use super::{fixtures::*, *};

#[test]
fn topology_health_report_flags_mixed_versions_and_unknown_joins() {
    let summary = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    let report = topology_health_report_from_summary(summary);

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.status, "attention");
    assert_eq!(report.registry_source_count, 5);
    assert_eq!(report.registry_version_min, Some(42));
    assert_eq!(report.registry_version_max, Some(46));
    assert!(!report.registry_versions_aligned);
    assert_eq!(report.stale_source_count, 0);
    assert_eq!(report.known_join_count, 8);
    assert_eq!(report.unknown_join_count, 5);
    assert_eq!(report.join_coverage, "61.5%");
}

#[test]
fn topology_health_text_renders_check_table() {
    let summary = topology_summary_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        subnet_report_fixture(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );
    let report = topology_health_report_from_summary(summary);

    let text = nns_topology_health_report_text(&report);

    assert!(text.contains("CHECK"));
    assert!(text.contains("registry_versions"));
    assert!(text.contains("attention"));
    assert!(text.contains("5 sources span registry versions 42..46"));
    assert!(text.contains("8 known, 5 unknown (61.5%)"));
}
