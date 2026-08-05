//! Module: nns::topology::report::text::check
//!
//! Responsibility: render NNS topology check reports as text.
//! Does not own: consistency-check derivation, cache loading, or JSON output.
//! Boundary: formats consistency-check rows and report status for humans.

use crate::{
    nns::topology::report::{NnsTopologyCheckReport, NnsTopologyCheckRow},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn nns_topology_check_report_text(report: &NnsTopologyCheckReport) -> String {
    render_check_table(&report.checks)
}

fn render_check_table(rows: &[NnsTopologyCheckRow]) -> String {
    let headers = ["CHECK", "STATUS", "DETAIL"];
    let rows = rows
        .iter()
        .map(|row| {
            [
                row.check.clone(),
                row.status.as_str().to_string(),
                row.detail.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left];
    render_table(&headers, &rows, &alignments)
}
