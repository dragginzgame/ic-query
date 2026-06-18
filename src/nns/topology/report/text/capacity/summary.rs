//! Module: nns::topology::report::text::capacity::summary
//!
//! Responsibility: render the NNS topology capacity summary table.
//! Does not own: capacity calculation, attention row filtering, or JSON output.
//! Boundary: formats aggregate capacity fields for humans.

use crate::{
    nns::topology::report::NnsTopologyCapacityReport,
    table::{ColumnAlign, render_table},
};

pub(super) fn render_capacity_summary_table(report: &NnsTopologyCapacityReport) -> String {
    let headers = ["FIELD", "VALUE"];
    let rows = [
        ["network".to_string(), report.network.clone()],
        ["status".to_string(), report.status.clone()],
        [
            "node_operators".to_string(),
            report.node_operator_count.to_string(),
        ],
        [
            "total_node_allowance".to_string(),
            report.total_node_allowance.to_string(),
        ],
        [
            "assigned_nodes".to_string(),
            report.assigned_node_count.to_string(),
        ],
        [
            "available_node_slots".to_string(),
            report.available_node_slots.to_string(),
        ],
        [
            "over_assigned_operators".to_string(),
            report.over_assigned_operator_count.to_string(),
        ],
        [
            "over_assigned_nodes".to_string(),
            report.over_assigned_node_count.to_string(),
        ],
        [
            "unknown_node_count_operators".to_string(),
            report.unknown_node_count_operator_count.to_string(),
        ],
    ];
    let alignments = [ColumnAlign::Left, ColumnAlign::Right];
    render_table(&headers, &rows, &alignments)
}
