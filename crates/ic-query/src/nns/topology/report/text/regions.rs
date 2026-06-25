//! Module: nns::topology::report::text::regions
//!
//! Responsibility: render NNS topology region reports as text.
//! Does not own: region aggregation, source reads, or JSON output.
//! Boundary: formats data-center region rows for human inspection.

use crate::{
    nns::topology::report::{NnsTopologyRegionRow, NnsTopologyRegionsReport},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn nns_topology_regions_report_text(report: &NnsTopologyRegionsReport) -> String {
    render_regions_table(&report.regions)
}

fn render_regions_table(rows: &[NnsTopologyRegionRow]) -> String {
    let headers = [
        "REGION",
        "DATA_CENTERS",
        "NODE_OPERATORS",
        "NODE_PROVIDERS",
        "NODES",
    ];
    let rows = rows
        .iter()
        .map(|row| {
            [
                row.region.clone(),
                row.data_center_count.to_string(),
                row.node_operator_count.to_string(),
                row.node_provider_count.to_string(),
                row.node_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Right,
    ];
    render_table(&headers, &rows, &alignments)
}
