use super::{fixtures::*, *};

#[test]
fn topology_capacity_report_summarizes_operator_allowance() {
    let report = topology_capacity_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_operator_report_fixture(),
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.status, NnsTopologyAssessmentStatus::Attention);
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
            && row.status == NnsTopologyCapacityStatus::Over
    }));
}

#[test]
fn topology_capacity_keeps_unknown_counts_and_operator_local_totals() {
    let mut operators = node_operator_report_fixture();
    let base = operators.node_operators[0].clone();
    operators.node_operators = [
        ("spare-b", 4, Some(1)),
        ("unknown", 8, None),
        ("over", 1, Some(2)),
        ("full", 1, Some(1)),
        ("spare-a", 4, Some(1)),
    ]
    .into_iter()
    .map(|(id, allowance, count)| {
        let mut row = base.clone();
        row.node_operator_principal = id.to_string();
        row.node_allowance = allowance;
        row.node_count = count;
        row
    })
    .collect();
    operators.node_operator_count = operators.node_operators.len();
    let report = topology_capacity_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        operators,
    );

    assert_eq!(report.status, NnsTopologyAssessmentStatus::Attention);
    assert_eq!(report.node_operator_count, 5);
    assert_eq!(report.total_node_allowance, 18);
    assert_eq!(report.assigned_node_count, 5);
    assert_eq!(report.unknown_node_count_operator_count, 1);
    assert_eq!(report.available_node_slots, 6);
    assert_eq!(report.over_assigned_operator_count, 1);
    assert_eq!(report.over_assigned_node_count, 1);
    assert_eq!(
        report
            .capacity
            .iter()
            .map(|row| row.node_operator_principal.as_str())
            .collect::<Vec<_>>(),
        ["over", "unknown", "full", "spare-a", "spare-b"]
    );
    let unknown = &report.capacity[1];
    assert_eq!(unknown.status, NnsTopologyCapacityStatus::Unknown);
    assert_eq!(unknown.assigned_node_count, None);
    assert_eq!(unknown.available_node_slots, None);
    assert_eq!(unknown.over_assigned_node_count, None);
    assert_eq!(unknown.utilization, "-");

    let mut empty = node_operator_report_fixture();
    empty.node_operators.clear();
    empty.node_operator_count = 0;
    let report = topology_capacity_report_from_report(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        empty,
    );
    assert_eq!(report.status, NnsTopologyAssessmentStatus::Ok);
    assert_eq!(report.capacity, []);
    assert_eq!(report.total_node_allowance, 0);
    assert_eq!(report.assigned_node_count, 0);
    assert_eq!(report.available_node_slots, 0);
    assert_eq!(report.over_assigned_node_count, 0);
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
