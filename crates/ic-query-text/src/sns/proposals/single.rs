//! Module: sns::proposals::single
//!
//! Responsibility: render one SNS proposal detail report as text.
//! Does not own: proposal fetching, cache lookup, report construction, or JSON output.
//! Boundary: formats one proposal DTO summary for humans.

use crate::sns::common::push_report_provenance_lines;
use crate::sns::proposals::SNS_PROPOSAL_DETAIL_TEXT_LIMIT;
use crate::sns::proposals::detail::proposal_detail_lines;
use crate::text_value::field;
use crate::text_value::yes_no;
use ic_query::sns::SnsProposalReport;

#[must_use]
/// Render the SNS proposal report as human-facing text.
pub fn sns_proposal_report_text(report: &SnsProposalReport, options: crate::TextOptions) -> String {
    let mut lines = crate::sns::common::sns_identity_lines(&report.context);
    lines.extend([
        field("proposal_id", &(report.proposal_id)),
        field("verbose", &(yes_no(options.verbose))),
    ]);
    push_report_provenance_lines(&mut lines, &report.provenance);
    lines.extend(crate::sns::common::sns_collection_lines(&report.context));
    lines.push(String::new());
    lines.push("proposal:".to_string());
    let detail_limit = (!options.verbose).then_some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT);
    lines.extend(proposal_detail_lines(&report.proposal, detail_limit));

    lines.join("\n")
}
