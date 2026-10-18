//! Module: sns::report::text::proposals::detail
//!
//! Responsibility: render shared SNS proposal detail fragments.
//! Does not own: proposal fetching, cache loading, report construction, or JSON output.
//! Boundary: formats proposal detail lines and ballot tables for proposal text reports.

use crate::sns::report::{sns_proposal_action_label, sns_proposal_vote_label};
use crate::subnet_catalog::format_utc_timestamp_secs;

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use super::SNS_PROPOSAL_TITLE_TEXT_LIMIT;
use crate::{
    sns::report::{
        SnsProposalBallotRow, SnsProposalRow,
        text::common::{neuron_id_text, optional_text, optional_timestamp_text},
    },
    text_value::{truncate_text, yes_no},
    token_amount::e8s_decimal_text,
};

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
    let action = sns_proposal_action_label(proposal.action_id);
    let mut lines = vec![
        format!("- proposal_id: {}", proposal.proposal_id),
        format!("  action_id: {}", proposal.action_id),
        format!("  action: {}", terminal_text(&action)),
        format!(
            "  decision_state: {}",
            terminal_text(proposal.decision_state.as_str())
        ),
        format!("  topic: {}", optional_text(proposal.topic.as_ref())),
        format!("  title: {}", terminal_text(&proposal.title)),
        format!("  url: {}", optional_text(proposal.url.as_ref())),
        format!(
            "  proposer_neuron_id: {}",
            optional_text(proposal.proposer_neuron_id.as_ref())
        ),
        format!(
            "  reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!(
            "  created_at: {}",
            format_utc_timestamp_secs(proposal.proposal_creation_timestamp_seconds)
        ),
        format!(
            "  decided_at: {}",
            optional_timestamp_text(proposal.decided_timestamp_seconds)
        ),
        format!(
            "  executed_at: {}",
            optional_timestamp_text(proposal.executed_timestamp_seconds)
        ),
        format!(
            "  failed_at: {}",
            optional_timestamp_text(proposal.failed_timestamp_seconds)
        ),
        format!("  reward_event_round: {}", proposal.reward_event_round),
        format!(
            "  is_eligible_for_rewards: {}",
            yes_no(proposal.is_eligible_for_rewards)
        ),
        format!("  ballot_count: {}", proposal.ballot_count),
    ];
    if let Some(tally) = proposal.latest_tally.as_ref() {
        lines.extend([
            format!("  tally_yes: {}", tally.yes),
            format!("  tally_no: {}", tally.no),
            format!("  tally_total: {}", tally.total),
        ]);
    }
    if let Some(reason) = proposal.failure_reason.as_ref() {
        lines.extend([
            format!("  failure_error_type: {}", reason.error_type),
            format!(
                "  failure_error_message: {}",
                terminal_text(&reason.error_message)
            ),
        ]);
    }
    if !proposal.summary.is_empty() {
        lines.push(format!(
            "  summary: {}",
            proposal_detail_text(&proposal.summary, detail_limit)
        ));
    }
    if let Some(rendering) = proposal.payload_text_rendering.as_ref() {
        lines.push(format!(
            "  payload_text_rendering: {}",
            proposal_detail_text(rendering, detail_limit)
        ));
    }
    lines
}

pub(super) fn proposal_ballot_table(
    ballots: &[SnsProposalBallotRow],
    verbose: bool,
) -> Option<String> {
    if ballots.is_empty() {
        return None;
    }
    Some(render(
        &["NEURON_ID", "VOTE", "VOTING_POWER", "CAST_AT"],
        &ballots
            .iter()
            .map(|ballot| {
                [
                    neuron_id_text(&ballot.neuron_id, verbose),
                    sns_proposal_vote_label(ballot.vote).into_owned(),
                    e8s_decimal_text(ballot.voting_power),
                    optional_timestamp_text(
                        (ballot.cast_timestamp_seconds != 0)
                            .then_some(ballot.cast_timestamp_seconds),
                    ),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            Alignment::Left,
            Alignment::Left,
            Alignment::Right,
            Alignment::Left,
        ],
        REPORT_TABLE_OPTIONS,
    ))
}

fn proposal_detail_text(value: &str, detail_limit: Option<usize>) -> String {
    detail_limit.map_or_else(
        || terminal_text(value).into_owned(),
        |limit| truncate_text(value, limit),
    )
}
