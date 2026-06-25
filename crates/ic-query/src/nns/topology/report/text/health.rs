//! Module: nns::topology::report::text::health
//!
//! Responsibility: render NNS topology health reports as text.
//! Does not own: health check derivation, cache loading, or JSON output.
//! Boundary: formats health check rows and report status for humans.

use crate::{
    nns::topology::report::{NnsTopologyHealthCheckRow, NnsTopologyHealthReport},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn nns_topology_health_report_text(report: &NnsTopologyHealthReport) -> String {
    render_health_check_table(&report.checks)
}

fn render_health_check_table(rows: &[NnsTopologyHealthCheckRow]) -> String {
    let headers = ["CHECK", "STATUS", "DETAIL"];
    let rows = rows
        .iter()
        .map(|row| [row.check.clone(), row.status.clone(), row.detail.clone()])
        .collect::<Vec<_>>();
    let alignments = [ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left];
    render_table(&headers, &rows, &alignments)
}
