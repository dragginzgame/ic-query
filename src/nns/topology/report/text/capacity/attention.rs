use crate::{
    nns::{render::compact_text, topology::report::NnsTopologyCapacityReport},
    table::{ColumnAlign, render_table},
};

use super::super::super::COMPACT_PRINCIPAL_CHARS;
use super::super::common::optional_u64_text;

pub(super) fn render_capacity_attention_table(report: &NnsTopologyCapacityReport) -> String {
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
