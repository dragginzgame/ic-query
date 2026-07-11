//! Module: nns::topology::report::text::refresh
//!
//! Responsibility: render NNS topology refresh reports as text.
//! Does not own: refresh execution, cache writing, or JSON output.
//! Boundary: formats component refresh outcomes for humans.

use crate::{
    nns::topology::report::NnsTopologyRefreshReport,
    table::{ColumnAlign, render_table},
    text_value::{sanitize_text, yes_no},
};

#[must_use]
pub fn nns_topology_refresh_report_text(report: &NnsTopologyRefreshReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "topology_refresh: {} components {} wrote {} replaced {} dry_run {}",
        sanitize_text(&report.network),
        report.component_count,
        report.wrote_cache_count,
        report.replaced_existing_cache_count,
        yes_no(report.dry_run)
    ));
    lines.push(format!(
        "source_endpoint: {}",
        sanitize_text(&report.source_endpoint)
    ));
    lines.push(render_refresh_table(report));
    lines.join("\n")
}

fn render_refresh_table(report: &NnsTopologyRefreshReport) -> String {
    let headers = [
        "SOURCE",
        "COUNT",
        "VERSION",
        "FETCHED_AT",
        "WROTE",
        "REPLACED",
        "CACHE",
    ];
    let rows = report
        .components
        .iter()
        .map(|row| {
            [
                row.source.clone(),
                row.item_count.to_string(),
                row.registry_version.to_string(),
                row.fetched_at.clone(),
                yes_no(row.wrote_cache).to_string(),
                yes_no(row.replaced_existing_cache).to_string(),
                row.cache_path.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}
