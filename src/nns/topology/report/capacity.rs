use super::{
    NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION, NnsTopologyCapacityReport, NnsTopologyCapacityRow,
};
use crate::nns::node_operator::report::{NnsNodeOperatorListReport, NnsNodeOperatorRow};

pub(super) fn topology_capacity_report_from_report(
    network: String,
    source_endpoint: String,
    node_operator_report: NnsNodeOperatorListReport,
) -> NnsTopologyCapacityReport {
    let mut capacity = node_operator_report
        .node_operators
        .iter()
        .map(capacity_row_from_operator)
        .collect::<Vec<_>>();
    capacity.sort_by(|left, right| {
        (
            capacity_status_rank(&left.status),
            left.available_node_slots.unwrap_or(u64::MAX),
            left.node_operator_principal.as_str(),
        )
            .cmp(&(
                capacity_status_rank(&right.status),
                right.available_node_slots.unwrap_or(u64::MAX),
                right.node_operator_principal.as_str(),
            ))
    });

    let total_node_allowance = capacity.iter().map(|row| row.node_allowance).sum();
    let assigned_node_count = capacity
        .iter()
        .filter_map(|row| row.assigned_node_count)
        .sum();
    let unknown_node_count_operator_count = capacity
        .iter()
        .filter(|row| row.assigned_node_count.is_none())
        .count();
    let available_node_slots = capacity
        .iter()
        .filter_map(|row| row.available_node_slots)
        .sum();
    let over_assigned_operator_count = capacity
        .iter()
        .filter(|row| row.over_assigned_node_count.is_some_and(|count| count > 0))
        .count();
    let over_assigned_node_count = capacity
        .iter()
        .filter_map(|row| row.over_assigned_node_count)
        .sum();
    let status = if over_assigned_operator_count == 0 && unknown_node_count_operator_count == 0 {
        "ok"
    } else {
        "attention"
    }
    .to_string();

    NnsTopologyCapacityReport {
        schema_version: NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status,
        node_operator_count: node_operator_report.node_operator_count,
        total_node_allowance,
        assigned_node_count,
        unknown_node_count_operator_count,
        available_node_slots,
        over_assigned_operator_count,
        over_assigned_node_count,
        capacity,
    }
}

fn capacity_row_from_operator(operator: &NnsNodeOperatorRow) -> NnsTopologyCapacityRow {
    let assigned_node_count = operator.node_count.map(u64::from);
    let available_node_slots =
        assigned_node_count.map(|node_count| operator.node_allowance.saturating_sub(node_count));
    let over_assigned_node_count =
        assigned_node_count.map(|node_count| node_count.saturating_sub(operator.node_allowance));
    let utilization = assigned_node_count.map_or_else(
        || "-".to_string(),
        |node_count| percent_text(node_count, operator.node_allowance),
    );
    let status = if over_assigned_node_count.is_some_and(|count| count > 0) {
        "over"
    } else if available_node_slots == Some(0) {
        "full"
    } else if available_node_slots.is_some() {
        "available"
    } else {
        "unknown"
    }
    .to_string();

    NnsTopologyCapacityRow {
        node_operator_principal: operator.node_operator_principal.clone(),
        node_provider_principal: operator.node_provider_principal.clone(),
        data_center_id: operator.data_center_id.clone(),
        node_allowance: operator.node_allowance,
        assigned_node_count,
        available_node_slots,
        over_assigned_node_count,
        utilization,
        status,
    }
}

fn capacity_status_rank(status: &str) -> u8 {
    match status {
        "over" => 0,
        "unknown" => 1,
        "full" => 2,
        "available" => 3,
        _ => 4,
    }
}

fn percent_text(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return "-".to_string();
    }
    let tenths = numerator
        .saturating_mul(1000)
        .saturating_add(denominator / 2)
        / denominator;
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}
