use super::SNS_PROPOSAL_TITLE_TEXT_LIMIT;
use crate::{
    nns::render::yes_no,
    sns::report::{
        SnsProposalBallotRow, SnsProposalRow,
        text::common::{neuron_id_text, optional_text, optional_u64_text, truncate_text_value},
    },
    table::{ColumnAlign, render_table},
    token_amount::e8s_decimal_text,
};

pub(super) fn proposal_title_for_list(proposal: &SnsProposalRow, verbose: bool) -> String {
    if verbose {
        proposal.title.clone()
    } else {
        truncate_text_value(&proposal.title, SNS_PROPOSAL_TITLE_TEXT_LIMIT)
    }
}

pub(super) fn proposal_detail_lines(
    proposal: &SnsProposalRow,
    detail_limit: Option<usize>,
) -> Vec<String> {
    let mut lines = vec![
        format!("- proposal_id: {}", optional_u64_text(proposal.proposal_id)),
        format!("  action_id: {}", proposal.action_id),
        format!("  action: {}", proposal.action),
        format!("  decision_state: {}", proposal.decision_state),
        format!("  title: {}", proposal.title),
        format!("  url: {}", optional_text(proposal.url.as_ref())),
        format!(
            "  proposer_neuron_id: {}",
            optional_text(proposal.proposer_neuron_id.as_ref())
        ),
        format!(
            "  reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!("  created_at: {}", proposal.created_at),
        format!(
            "  decided_at: {}",
            optional_text(proposal.decided_at.as_ref())
        ),
        format!(
            "  executed_at: {}",
            optional_text(proposal.executed_at.as_ref())
        ),
        format!(
            "  failed_at: {}",
            optional_text(proposal.failed_at.as_ref())
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
            format!("  failure_error_message: {}", reason.error_message),
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
    Some(render_table(
        &["NEURON_ID", "VOTE", "VOTING_POWER", "CAST_AT"],
        &ballots
            .iter()
            .map(|ballot| {
                [
                    neuron_id_text(&ballot.neuron_id, verbose),
                    ballot.vote_text.clone(),
                    e8s_decimal_text(ballot.voting_power),
                    optional_text(ballot.cast_at.as_ref()).to_string(),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Right,
            ColumnAlign::Left,
        ],
    ))
}

fn proposal_detail_text(value: &str, detail_limit: Option<usize>) -> String {
    detail_limit.map_or_else(
        || value.to_string(),
        |limit| truncate_text_value(value, limit),
    )
}
