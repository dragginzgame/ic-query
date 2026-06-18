//! Module: sns::report::live::convert::proposals
//!
//! Responsibility: convert SNS governance proposal wire rows.
//! Does not own: governance transport, proposal request construction, or rendering.
//! Boundary: maps live proposal data into report rows used by text and JSON output.

mod ballot;
mod labels;
mod timestamp;

use super::common::clean_optional_text;
use crate::sns::report::{
    SnsProposalFailureReason, SnsProposalRow, SnsProposalTally, hex_bytes,
    live::types::SnsGovernanceProposalData,
};
use ballot::sns_proposal_ballot_row;
use labels::proposal_action_text;
use timestamp::{nonzero_timestamp, optional_timestamp_text};

/// Convert one SNS governance proposal wire row into a report row.
pub(in crate::sns::report::live) fn sns_proposal_row(
    proposal: SnsGovernanceProposalData,
) -> SnsProposalRow {
    let decision_state = proposal_decision_state(&proposal);
    let proposal_id = proposal.id.as_ref().map(|id| id.id);
    let proposal_fields = proposal.proposal.unwrap_or_default();
    let ballots = proposal
        .ballots
        .into_iter()
        .map(sns_proposal_ballot_row)
        .collect::<Vec<_>>();
    let ballot_count = ballots.len();
    SnsProposalRow {
        proposal_id,
        action_id: proposal.action,
        action: proposal_action_text(proposal.action),
        title: proposal_fields.title,
        summary: proposal_fields.summary,
        url: clean_optional_text(Some(proposal_fields.url)),
        decision_state,
        reject_cost_e8s: proposal.reject_cost_e8s,
        proposal_creation_timestamp_seconds: proposal.proposal_creation_timestamp_seconds,
        created_at: crate::subnet_catalog::format_utc_timestamp_secs(
            proposal.proposal_creation_timestamp_seconds,
        ),
        decided_timestamp_seconds: nonzero_timestamp(proposal.decided_timestamp_seconds),
        decided_at: optional_timestamp_text(proposal.decided_timestamp_seconds),
        executed_timestamp_seconds: nonzero_timestamp(proposal.executed_timestamp_seconds),
        executed_at: optional_timestamp_text(proposal.executed_timestamp_seconds),
        failed_timestamp_seconds: nonzero_timestamp(proposal.failed_timestamp_seconds),
        failed_at: optional_timestamp_text(proposal.failed_timestamp_seconds),
        failure_reason: proposal
            .failure_reason
            .map(|reason| SnsProposalFailureReason {
                error_type: reason.error_type,
                error_message: reason.error_message,
            }),
        reward_event_round: proposal.reward_event_round,
        reward_event_end_timestamp_seconds: proposal.reward_event_end_timestamp_seconds,
        is_eligible_for_rewards: proposal.is_eligible_for_rewards,
        latest_tally: proposal.latest_tally.map(|tally| SnsProposalTally {
            timestamp_seconds: tally.timestamp_seconds,
            yes: tally.yes,
            no: tally.no,
            total: tally.total,
        }),
        ballot_count,
        ballots,
        payload_text_rendering: proposal
            .payload_text_rendering
            .and_then(|value| clean_optional_text(Some(value))),
        proposer_neuron_id: proposal.proposer.map(|id| hex_bytes(&id.id)),
    }
}

fn proposal_decision_state(proposal: &SnsGovernanceProposalData) -> String {
    if proposal.failed_timestamp_seconds > 0 {
        "failed"
    } else if proposal.executed_timestamp_seconds > 0 {
        "executed"
    } else if proposal.decided_timestamp_seconds > 0 {
        "decided"
    } else {
        "open"
    }
    .to_string()
}
