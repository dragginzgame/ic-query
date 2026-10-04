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
    let mut report = NnsTopologyCapacityReport {
        schema_version: NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status: NnsTopologyAssessmentStatus::Ok,
        node_operator_count: node_operator_report.node_operator_count,
        total_node_allowance: 0,
        assigned_node_count: 0,
        unknown_node_count_operator_count: 0,
        available_node_slots: 0,
        over_assigned_operator_count: 0,
        over_assigned_node_count: 0,
        capacity: Vec::with_capacity(node_operator_report.node_operators.len()),
    };
    for operator in node_operator_report.node_operators {
        let row = capacity_row_from_operator(operator);
        report.total_node_allowance += row.node_allowance;
        report.assigned_node_count += row.assigned_node_count.unwrap_or(0);
        report.unknown_node_count_operator_count += usize::from(row.assigned_node_count.is_none());
        report.available_node_slots += row.available_node_slots.unwrap_or(0);
        report.over_assigned_operator_count +=
            usize::from(row.over_assigned_node_count.is_some_and(|count| count > 0));
        report.over_assigned_node_count += row.over_assigned_node_count.unwrap_or(0);
        report.capacity.push(row);
    }
    report.status = NnsTopologyAssessmentStatus::from_ok(
        report.over_assigned_operator_count == 0 && report.unknown_node_count_operator_count == 0,
    );
    sort_capacity_rows(&mut report.capacity);
    report
}

fn capacity_row_from_operator(operator: NnsNodeOperatorRow) -> NnsTopologyCapacityRow {
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
        node_operator_principal: operator.node_operator_principal,
        node_provider_principal: operator.node_provider_principal,
        data_center_id: operator.data_center_id,
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
