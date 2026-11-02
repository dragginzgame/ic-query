//! Module: sns::proposals::single
//!
//! Responsibility: render one SNS proposal detail report as text.
//! Does not own: proposal fetching, cache lookup, report construction, or JSON output.
//! Boundary: formats one proposal DTO and optional ballot table for humans.

use crate::sns::common::push_report_provenance_lines;
use crate::sns::proposals::SNS_PROPOSAL_DETAIL_TEXT_LIMIT;
use crate::sns::proposals::detail::proposal_ballot_table;
use crate::sns::proposals::detail::proposal_detail_lines;
use crate::text_value::yes_no;
use ic_query::sns::SnsProposalReport;

#[must_use]
/// Render the SNS proposal report as human-facing text.
pub fn sns_proposal_report_text(report: &SnsProposalReport) -> String {
    let mut lines = crate::sns::common::sns_identity_lines(&report.context);
    lines.extend([
        format!("proposal_id: {}", report.proposal_id),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("show_ballots: {}", yes_no(report.show_ballots)),
    ]);
    push_report_provenance_lines(&mut lines, &report.provenance);
    lines.extend(crate::sns::common::sns_collection_lines(&report.context));
    lines.push(String::new());
    lines.push("proposal:".to_string());
    let detail_limit = (!report.verbose).then_some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT);
    lines.extend(proposal_detail_lines(&report.proposal, detail_limit));
    if report.show_ballots {
        lines.push(String::new());
        lines.push("ballots:".to_string());
        if let Some(table) = proposal_ballot_table(&report.proposal.ballots, report.verbose) {
            lines.push(table);
        } else {
            lines.push("-".to_string());
        }
    }
    lines.join("\n")
}
