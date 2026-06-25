//! Module: nns::topology::report::text::capacity
//!
//! Responsibility: render NNS topology capacity reports as text.
//! Does not own: capacity calculation, source reads, or JSON output.
//! Boundary: combines summary and attention capacity tables for humans.

use crate::{
    nns::{
        render::compact_text,
        topology::report::{
            COMPACT_PRINCIPAL_CHARS, NnsTopologyCapacityReport, text::common::optional_u64_text,
        },
    },
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn nns_topology_capacity_report_text(report: &NnsTopologyCapacityReport) -> String {
    let lines = [
        render_capacity_summary_table(report),
        String::new(),
        render_capacity_attention_table(report),
    ];
    lines.join("\n")
}

fn render_capacity_summary_table(report: &NnsTopologyCapacityReport) -> String {
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

fn render_capacity_attention_table(report: &NnsTopologyCapacityReport) -> String {
    let attention_rows = report
        .capacity
        .iter()
        .filter(|row| matches!(row.status.as_str(), "over" | "unknown"))
        .collect::<Vec<_>>();
    if attention_rows.is_empty() {
        let headers = ["STATUS", "DETAIL"];
        let rows = [[
            report.status.clone(),
            "no capacity attention rows".to_string(),
        ]];
        let alignments = [ColumnAlign::Left, ColumnAlign::Left];
        return render_table(&headers, &rows, &alignments);
    }

    let headers = [
        "NODE_OPERATOR",
        "NODE_PROVIDER",
        "DATA_CENTER",
        "ALLOWANCE",
        "NODES",
        "AVAILABLE",
        "OVER",
        "UTILIZATION",
        "STATUS",
    ];
    let rows = attention_rows
        .iter()
        .map(|row| {
            [
                compact_text(&row.node_operator_principal, COMPACT_PRINCIPAL_CHARS),
                compact_text(&row.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                row.data_center_id.clone(),
                row.node_allowance.to_string(),
                optional_u64_text(row.assigned_node_count),
                optional_u64_text(row.available_node_slots),
                optional_u64_text(row.over_assigned_node_count),
                row.utilization.clone(),
                row.status.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}
