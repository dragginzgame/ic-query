//! Module: nns::topology::report::text::gaps
//!
//! Responsibility: render NNS topology gap reports as text.
//! Does not own: gap detection, source reads, or JSON output.
//! Boundary: formats known topology relation gaps for human inspection.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::table::{Alignment, render};

use crate::nns::topology::report::{NnsTopologyGapRow, NnsTopologyGapsReport};

#[must_use]
pub fn nns_topology_gaps_report_text(report: &NnsTopologyGapsReport) -> String {
    if report.gaps.is_empty() {
        return render_gaps_status_table(report);
    }
    render_gaps_table(&report.gaps)
}

fn render_gaps_status_table(report: &NnsTopologyGapsReport) -> String {
    let headers = ["STATUS", "DETAIL"];
    let rows = [[
        report.status.as_str().to_string(),
        "no topology join gaps".to_string(),
    ]];
    let alignments = [Alignment::Left, Alignment::Left];
    render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS)
}

fn render_gaps_table(rows: &[NnsTopologyGapRow]) -> String {
    let headers = [
        "SUBJECT_KIND",
        "SUBJECT",
        "MISSING_RELATION",
        "REFERENCED_ID",
    ];
    let rows = rows
        .iter()
        .map(|row| {
            [
                row.subject_kind.as_str().to_string(),
                row.subject.clone(),
                row.missing_relation.as_str().to_string(),
                row.referenced_id.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
    ];
    render(&headers, &rows, &alignments, REPORT_TABLE_OPTIONS)
}
