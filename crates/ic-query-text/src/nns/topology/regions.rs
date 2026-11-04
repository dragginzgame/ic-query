//! Module: nns::topology::regions
//!
//! Responsibility: render NNS topology region reports as text.
//! Does not own: region aggregation, source reads, or JSON output.
//! Boundary: formats data-center region rows for human inspection.

use crate::text_value::table;
use ic_host_text::table::Alignment;

use ic_query::nns::topology::NnsTopologyRegionRow;
use ic_query::nns::topology::NnsTopologyRegionsReport;

#[must_use]
/// Render the NNS topology regions report as human-facing text.
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
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
    ];
    table(&headers, &rows, &alignments)
}
