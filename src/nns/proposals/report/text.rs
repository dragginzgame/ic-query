//! Module: nns::proposals::report::text
//!
//! Responsibility: render NNS proposal reports as text.
//! Does not own: live governance calls, JSON output, or report assembly.
//! Boundary: formats NNS proposal rows for human CLI output.

use super::model::{NnsProposalReport, NnsProposalRow, NnsProposalsReport};
use crate::{
    table::{ColumnAlign, render_table},
    token_amount::e8s_decimal_text,
};

#[must_use]
pub(in crate::nns::proposals) fn nns_proposals_report_text(report: &NnsProposalsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "before_proposal_id: {}",
            optional_u64_text(report.before_proposal_id)
        ),
        format!("status_filter: {}", report.status_filter),
        format!("topic_filter: {}", report.topic_filter),
        format!("sort: {}", report.sort),
        format!("sort_direction: {}", report.sort_direction),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("proposal_count: {}", report.proposal_count),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
    ];
    if !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["ID", "TOPIC", "STATUS", "PROPOSED_AT", "TITLE"],
            &report
                .proposals
                .iter()
                .map(|proposal| {
                    [
                        optional_u64_text(proposal.proposal_id),
                        proposal.topic_text.clone(),
                        proposal.status_text.clone(),
                        proposal.proposed_at.clone(),
                        proposal_title(proposal),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    if report.verbose && !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push("proposal_details:".to_string());
        for proposal in &report.proposals {
            lines.extend(proposal_detail_lines(proposal));
        }
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::nns::proposals) fn nns_proposal_report_text(report: &NnsProposalReport) -> String {
    let proposal = &report.proposal;
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("proposal_id: {}", report.proposal_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
        String::new(),
    ];
    lines.extend(proposal_detail_lines(proposal));
    lines.join("\n")
}

fn proposal_detail_lines(proposal: &NnsProposalRow) -> Vec<String> {
    let tally = proposal.latest_tally.as_ref();
    vec![
        format!("proposal_id: {}", optional_u64_text(proposal.proposal_id)),
        format!(
            "proposer_neuron_id: {}",
            optional_u64_text(proposal.proposer_neuron_id)
        ),
        format!("topic: {} ({})", proposal.topic_text, proposal.topic),
        format!("status: {} ({})", proposal.status_text, proposal.status),
        format!(
            "reward_status: {} ({})",
            proposal.reward_status_text, proposal.reward_status
        ),
        format!("action: {}", proposal.action_text.as_deref().unwrap_or("-")),
        format!("title: {}", proposal_title(proposal)),
        format!("url: {}", empty_text(&proposal.url)),
        format!(
            "reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!("proposed_at: {}", proposal.proposed_at),
        format!(
            "deadline_at: {}",
            proposal.deadline_at.as_deref().unwrap_or("-")
        ),
        format!(
            "decided_at: {}",
            proposal.decided_at.as_deref().unwrap_or("-")
        ),
        format!(
            "executed_at: {}",
            proposal.executed_at.as_deref().unwrap_or("-")
        ),
        format!(
            "failed_at: {}",
            proposal.failed_at.as_deref().unwrap_or("-")
        ),
        format!("reward_event_round: {}", proposal.reward_event_round),
        format!(
            "total_potential_voting_power: {}",
            optional_u64_text(proposal.total_potential_voting_power)
        ),
        format!("ballot_count: {}", proposal.ballot_count),
        format!(
            "latest_tally_yes: {}",
            tally.map_or_else(|| "-".to_string(), |tally| tally.yes.to_string())
        ),
        format!(
            "latest_tally_no: {}",
            tally.map_or_else(|| "-".to_string(), |tally| tally.no.to_string())
        ),
        format!("summary: {}", empty_text(&proposal.summary)),
    ]
}

fn proposal_title(proposal: &NnsProposalRow) -> String {
    proposal
        .title
        .as_ref()
        .filter(|title| !title.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "-".to_string())
}

fn empty_text(value: &str) -> &str {
    if value.trim().is_empty() { "-" } else { value }
}

fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
