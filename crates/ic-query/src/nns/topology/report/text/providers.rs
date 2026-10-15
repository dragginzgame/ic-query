//! Module: nns::topology::report::text::providers
//!
//! Responsibility: render NNS topology provider reports as text.
//! Does not own: provider aggregation, source reads, or JSON output.
//! Boundary: formats provider distribution rows for human inspection.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::table::{Alignment, render};

use super::common::render_registry_version_table;
use crate::{
    nns::{
        render::compact_text,
        topology::report::{
            COMPACT_PRINCIPAL_CHARS, NnsTopologyProviderRow, NnsTopologyProvidersReport,
        },
    },
    text_value::optional_u64_text,
};

#[must_use]
pub fn nns_topology_providers_report_text(report: &NnsTopologyProvidersReport) -> String {
    format!(
        "{}\n\n{}",
        render_providers_table(&report.providers),
        render_registry_version_table(&report.registry_versions)
    )
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
                row.status.as_str().to_string(),
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
        Alignment::Left,
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
    ];
    render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS)
}
