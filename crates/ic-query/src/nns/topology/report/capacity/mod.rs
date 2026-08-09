//! Module: nns::topology::report::capacity
//!
//! Responsibility: build NNS topology capacity reports.
//! Does not own: source report loading, text rendering, or command parsing.
//! Boundary: maps node-operator rows into sorted capacity report rows.

use super::{
    NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION, NnsTopologyAssessmentStatus,
    NnsTopologyCapacityReport, NnsTopologyCapacityRow, NnsTopologyCapacityStatus,
    percent::ratio_percent_text,
};
use crate::nns::node_operator::{NnsNodeOperatorListReport, NnsNodeOperatorRow};

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
    sort_capacity_rows(&mut capacity);

    let summary = capacity_summary(&capacity);

    NnsTopologyCapacityReport {
        schema_version: NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status: summary.status,
        node_operator_count: node_operator_report.node_operator_count,
        total_node_allowance: summary.total_node_allowance,
        assigned_node_count: summary.assigned_node_count,
        unknown_node_count_operator_count: summary.unknown_node_count_operator_count,
        available_node_slots: summary.available_node_slots,
        over_assigned_operator_count: summary.over_assigned_operator_count,
        over_assigned_node_count: summary.over_assigned_node_count,
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
        |node_count| {
            ratio_percent_text(u128::from(node_count), u128::from(operator.node_allowance))
        },
    );
    let status = if over_assigned_node_count.is_some_and(|count| count > 0) {
        NnsTopologyCapacityStatus::Over
    } else if available_node_slots == Some(0) {
        NnsTopologyCapacityStatus::Full
    } else if available_node_slots.is_some() {
        NnsTopologyCapacityStatus::Available
    } else {
        NnsTopologyCapacityStatus::Unknown
    };

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

fn sort_capacity_rows(capacity: &mut [NnsTopologyCapacityRow]) {
    capacity.sort_by(|left, right| {
        (
            left.status.sort_rank(),
            left.available_node_slots.unwrap_or(u64::MAX),
            left.node_operator_principal.as_str(),
        )
            .cmp(&(
                right.status.sort_rank(),
                right.available_node_slots.unwrap_or(u64::MAX),
                right.node_operator_principal.as_str(),
            ))
    });
}

struct CapacitySummary {
    status: NnsTopologyAssessmentStatus,
    total_node_allowance: u64,
    assigned_node_count: u64,
    unknown_node_count_operator_count: usize,
    available_node_slots: u64,
    over_assigned_operator_count: usize,
    over_assigned_node_count: u64,
}

fn capacity_summary(capacity: &[NnsTopologyCapacityRow]) -> CapacitySummary {
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
    let status = NnsTopologyAssessmentStatus::from_ok(
        over_assigned_operator_count == 0 && unknown_node_count_operator_count == 0,
    );

    CapacitySummary {
        status,
        total_node_allowance,
        assigned_node_count,
        unknown_node_count_operator_count,
        available_node_slots,
        over_assigned_operator_count,
        over_assigned_node_count,
    }
}
