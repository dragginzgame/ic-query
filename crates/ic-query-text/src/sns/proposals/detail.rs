//! Module: sns::proposals::detail
//!
//! Responsibility: render shared SNS proposal detail fragments.
//! Does not own: proposal fetching, cache loading, report construction, or JSON output.
//! Boundary: formats concise proposal summaries for proposal text reports.

use crate::text_value::field;
use ic_host_text::timestamp::format_utc_timestamp_secs;
use ic_query::sns::sns_proposal_action_label;

use ic_host_text::escape::terminal_text;

use crate::sns::proposals::SNS_PROPOSAL_TITLE_TEXT_LIMIT;
use crate::text_value::truncate_text;
use ic_query::sns::SnsProposalRow;

pub(super) fn proposal_title_for_list(proposal: &SnsProposalRow, verbose: bool) -> String {
    if verbose {
        proposal.title.clone()
    } else {
        truncate_text(&proposal.title, SNS_PROPOSAL_TITLE_TEXT_LIMIT)
    }
}

pub(super) fn proposal_detail_lines(
    proposal: &SnsProposalRow,
    detail_limit: Option<usize>,
) -> Vec<String> {
    vec![
        field("proposal_id", &(proposal.proposal_id)),
        field("action", &(sns_proposal_action_label(proposal.action_id))),
        field("decision_state", &(proposal.decision_state().as_str())),
        field("title", &proposal.title),
        field(
            "created_at",
            &(format_utc_timestamp_secs(proposal.proposal_creation_timestamp_seconds)),
        ),
        field(
            "summary",
            &(proposal_detail_text(&proposal.summary, detail_limit)),
        ),
    ]
}

fn proposal_detail_text(value: &str, detail_limit: Option<usize>) -> String {
    detail_limit.map_or_else(
        || terminal_text(value).into_owned(),
        |limit| truncate_text(value, limit),
    )
}
