//! Module: nns::topology::report::text::check
//!
//! Responsibility: render NNS topology check reports as text.
//! Does not own: consistency-check derivation, cache loading, or JSON output.
//! Boundary: formats consistency-check rows and report status for humans.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::table::{Alignment, render};

use crate::nns::topology::report::{NnsTopologyCheckReport, NnsTopologyCheckRow};

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
    let alignments = [Alignment::Left, Alignment::Left, Alignment::Left];
    render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS)
}
