use super::{fixtures::*, *};

#[test]
fn topology_gaps_report_lists_unknown_join_subjects() {
    let report = topology_gaps_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.status, "attention");
    assert_eq!(report.gap_count, 5);
    assert!(report.gaps.iter().any(|gap| {
        gap.subject_kind == "node"
            && gap.subject == "node-c"
            && gap.missing_relation == "node_provider"
            && gap.referenced_id == "provider-z"
    }));
    assert!(report.gaps.iter().any(|gap| {
        gap.subject_kind == "node"
            && gap.subject == "node-c"
            && gap.missing_relation == "node_operator"
            && gap.referenced_id == "operator-z"
    }));
    assert!(report.gaps.iter().any(|gap| {
        gap.subject_kind == "node_operator"
            && gap.subject == "operator-b"
            && gap.missing_relation == "data_center"
            && gap.referenced_id == "dc-z"
    }));
}

#[test]
fn topology_gaps_text_renders_gap_or_ok_tables() {
    let report = topology_gaps_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    let text = nns_topology_gaps_report_text(&report);

    assert!(text.contains("SUBJECT_KIND"));
    assert!(text.contains("MISSING_RELATION"));
    assert!(text.contains("node-c"));
    assert!(text.contains("provider-z"));

    let clean_report = topology_gaps_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_report_fixture(),
        complete_node_provider_report_fixture(),
        complete_node_operator_report_fixture(),
        complete_data_center_report_fixture(),
    );
    let clean_text = nns_topology_gaps_report_text(&clean_report);

    assert_eq!(clean_report.status, "ok");
    assert_eq!(clean_report.gap_count, 0);
    assert!(clean_text.contains("STATUS"));
    assert!(clean_text.contains("no topology join gaps"));
}
