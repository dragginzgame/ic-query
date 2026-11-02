//! Module: sns::proposals::list
//!
//! Responsibility: render SNS proposal list reports as text.
//! Does not own: proposal fetching, cache filtering, report construction, or JSON output.
//! Boundary: formats live or cache-backed proposal rows for humans.

use crate::sns::common::sns_identity_lines;

use ic_host_text::timestamp::format_utc_timestamp_secs;
use ic_query::sns::sns_proposal_action_label;

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;
use ic_host_text::table::render;

use crate::sns::common::push_report_provenance_lines;
use crate::sns::proposals::SNS_PROPOSAL_DETAIL_TEXT_LIMIT;
use crate::sns::proposals::detail::proposal_detail_lines;
use crate::sns::proposals::detail::proposal_title_for_list;
use crate::text_value::optional_u64_text;
use crate::text_value::yes_no;
use ic_query::sns::SnsProposalsReport;

#[must_use]
/// Render the SNS proposals report as human-facing text.
pub fn sns_proposals_report_text(
    report: &SnsProposalsReport,
    options: crate::TextOptions,
) -> String {
    let mut lines = sns_identity_lines(&report.context);
    lines.extend([
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "before_proposal_id: {}",
            optional_u64_text(report.before_proposal_id)
        ),
        format!("status_filter: {}", report.status_filter),
        format!("topic_filter: {}", report.topic_filter),
        format!("eligibility_filter: {}", report.eligibility_filter),
        format!(
            "proposer_filter: {}",
            terminal_text(report.proposer_filter.as_deref().unwrap_or("-"))
        ),
        format!(
            "query_filter: {}",
            terminal_text(report.query_filter.as_deref().unwrap_or("-"))
        ),
        format!("sort: {}", report.sort),
        format!("sort_direction: {}", report.sort_direction),
        format!("verbose: {}", yes_no(options.verbose)),
    ]);
    push_report_provenance_lines(&mut lines, &report.provenance);
    lines.extend([format!("proposal_count: {}", report.proposal_count)]);
    lines.extend(crate::sns::common::sns_collection_lines(&report.context));
    if !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push(render(
            &["ID", "ACTION", "DECISION", "CREATED_AT", "TITLE"],
            &report
                .proposals
                .iter()
                .map(|proposal| {
                    [
                        proposal.proposal_id.to_string(),
                        sns_proposal_action_label(proposal.action_id).into_owned(),
                        proposal.decision_state().as_str().to_string(),
                        format_utc_timestamp_secs(proposal.proposal_creation_timestamp_seconds),
                        proposal_title_for_list(proposal, options.verbose),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Right,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
            ],
            REPORT_TABLE_OPTIONS,
        ));
    }
    if options.verbose && !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push("proposal_details:".to_string());
        for proposal in &report.proposals {
            lines.extend(proposal_detail_lines(
                proposal,
                Some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT),
            ));
        }
    }
    lines.join("\n")
}
