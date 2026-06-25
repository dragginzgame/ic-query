//! Module: nns::topology::report::capacity
//!
//! Responsibility: build NNS topology capacity reports.
//! Does not own: source report loading, text rendering, or command parsing.
//! Boundary: maps node-operator rows into sorted capacity report rows.

mod row;
mod sort;
mod summary;

use super::{NNS_TOPOLOGY_CAPACITY_REPORT_SCHEMA_VERSION, NnsTopologyCapacityReport};
use crate::nns::node_operator::report::NnsNodeOperatorListReport;
use row::capacity_row_from_operator;
use sort::sort_capacity_rows;
use summary::capacity_summary;

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
