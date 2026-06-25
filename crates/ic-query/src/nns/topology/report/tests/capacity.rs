use super::{fixtures::*, *};

#[test]
fn topology_capacity_report_summarizes_operator_allowance() {
    let report = topology_capacity_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_operator_report_fixture(),
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.status, "attention");
    assert_eq!(report.node_operator_count, 2);
    assert_eq!(report.total_node_allowance, 2);
    assert_eq!(report.assigned_node_count, 3);
    assert_eq!(report.available_node_slots, 0);
    assert_eq!(report.over_assigned_operator_count, 1);
    assert_eq!(report.over_assigned_node_count, 1);
    assert!(report.capacity.iter().any(|row| {
        row.node_operator_principal == "operator-a"
            && row.assigned_node_count == Some(2)
            && row.over_assigned_node_count == Some(1)
            && row.utilization == "200.0%"
            && row.status == "over"
    }));
}

#[test]
fn topology_capacity_text_renders_operator_capacity_table() {
    let report = topology_capacity_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_operator_report_fixture(),
    );

    let text = nns_topology_capacity_report_text(&report);

    assert!(text.contains("NODE_OPERATOR"));
    assert!(text.contains("ALLOWANCE"));
    assert!(text.contains("UTILIZATION"));
    assert!(text.contains("operator-a"));
    assert!(text.contains("200.0%"));
}
