use super::super::{COMPACT_PRINCIPAL_CHARS, NnsTopologyProviderRow, NnsTopologyProvidersReport};
use super::common::optional_u64_text;
use crate::{
    nns::render::compact_text,
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn nns_topology_providers_report_text(report: &NnsTopologyProvidersReport) -> String {
    render_providers_table(&report.providers)
}

fn render_providers_table(rows: &[NnsTopologyProviderRow]) -> String {
    let headers = [
        "NODE_PROVIDER",
        "STATUS",
        "GOV_NODES",
        "NODES",
        "OPERATORS",
        "DATA_CENTERS",
        "REGIONS",
        "ALLOWANCE",
        "AVAILABLE",
        "OVER",
    ];
    let rows = rows
        .iter()
        .map(|row| {
            [
                compact_text(&row.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                row.status.clone(),
                optional_u64_text(row.governance_node_count),
                row.topology_node_count.to_string(),
                row.node_operator_count.to_string(),
                row.data_center_count.to_string(),
                row.region_count.to_string(),
                row.total_node_allowance.to_string(),
                row.available_node_slots.to_string(),
                row.over_assigned_node_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
    ];
    render_table(&headers, &rows, &alignments)
}
