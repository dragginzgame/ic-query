use super::super::{NnsTopologyGapRow, NnsTopologyGapsReport};
use crate::table::{ColumnAlign, render_table};

#[must_use]
pub fn nns_topology_gaps_report_text(report: &NnsTopologyGapsReport) -> String {
    if report.gaps.is_empty() {
        return render_gaps_status_table(report);
    }
    render_gaps_table(&report.gaps)
}

fn render_gaps_status_table(report: &NnsTopologyGapsReport) -> String {
    let headers = ["STATUS", "DETAIL"];
    let rows = [[report.status.clone(), "no topology join gaps".to_string()]];
    let alignments = [ColumnAlign::Left, ColumnAlign::Left];
    render_table(&headers, &rows, &alignments)
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
                row.subject_kind.clone(),
                row.subject.clone(),
                row.missing_relation.clone(),
                row.referenced_id.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
        ColumnAlign::Left,
    ];
    render_table(&headers, &rows, &alignments)
}
