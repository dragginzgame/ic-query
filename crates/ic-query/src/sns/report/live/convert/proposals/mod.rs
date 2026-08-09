//! Module: sns::report::live::convert::proposals
//!
//! Responsibility: convert SNS governance proposal wire rows.
//! Does not own: governance transport, proposal request construction, or rendering.
//! Boundary: maps live proposal data into report rows used by text and JSON output.

use super::common::clean_optional_text;
use crate::{
    sns::report::{
        SnsHostError, SnsProposalAction, SnsProposalBallotRow, SnsProposalDecisionState,
        SnsProposalFailureReason, SnsProposalRow, SnsProposalTally, SnsProposalVote, hex_bytes,
        live::types::{SnsGovernanceBallot, SnsGovernanceProposalData, SnsTopic},
    },
    subnet_catalog::format_utc_timestamp_secs,
};

/// Convert one SNS governance proposal wire row into a report row.
pub(in crate::sns::report::live) fn sns_proposal_row(
    proposal: SnsGovernanceProposalData,
) -> Result<SnsProposalRow, SnsHostError> {
    let decision_state = proposal_decision_state(&proposal);
    let proposal_id = proposal
        .id
        .as_ref()
        .ok_or(SnsHostError::MissingProposalId)?
        .id;
    let proposal_fields = proposal.proposal.unwrap_or_default();
    let ballots = proposal
        .ballots
        .into_iter()
        .map(sns_proposal_ballot_row)
        .collect::<Vec<_>>();
    let ballot_count = ballots.len();
    Ok(SnsProposalRow {
        proposal_id,
        action_id: proposal.action,
        action: SnsProposalAction::from_id(proposal.action),
        title: proposal_fields.title,
        summary: proposal_fields.summary,
        url: clean_optional_text(Some(proposal_fields.url)),
        decision_state,
        status: Some(proposal.status),
        topic: proposal
            .topic
            .map(|topic| sns_topic_text(topic).to_string()),
        reject_cost_e8s: proposal.reject_cost_e8s,
        proposal_creation_timestamp_seconds: proposal.proposal_creation_timestamp_seconds,
        created_at: format_utc_timestamp_secs(proposal.proposal_creation_timestamp_seconds),
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
    })
}

/// Convert an SNS Governance topic into its stable report label.
pub(super) const fn sns_topic_text(topic: SnsTopic) -> &'static str {
    match topic {
        SnsTopic::DaoCommunitySettings => "dao-community-settings",
        SnsTopic::SnsFrameworkManagement => "sns-framework-management",
        SnsTopic::DappCanisterManagement => "dapp-canister-management",
        SnsTopic::ApplicationBusinessLogic => "application-business-logic",
        SnsTopic::Governance => "governance",
        SnsTopic::TreasuryAssetManagement => "treasury-asset-management",
        SnsTopic::CriticalDappOperations => "critical-dapp-operations",
    }
}

const fn proposal_decision_state(proposal: &SnsGovernanceProposalData) -> SnsProposalDecisionState {
    if proposal.failed_timestamp_seconds > 0 {
        SnsProposalDecisionState::Failed
    } else if proposal.executed_timestamp_seconds > 0 {
        SnsProposalDecisionState::Executed
    } else if proposal.decided_timestamp_seconds > 0 {
        SnsProposalDecisionState::Decided
    } else {
        SnsProposalDecisionState::Open
    }
}

fn sns_proposal_ballot_row(
    (neuron_id, ballot): (String, SnsGovernanceBallot),
) -> SnsProposalBallotRow {
    SnsProposalBallotRow {
        neuron_id,
        vote: ballot.vote,
        vote_text: SnsProposalVote::from_code(ballot.vote),
        cast_timestamp_seconds: ballot.cast_timestamp_seconds,
        cast_at: optional_timestamp_text(ballot.cast_timestamp_seconds),
        voting_power: ballot.voting_power,
    }
}

const fn nonzero_timestamp(timestamp_seconds: u64) -> Option<u64> {
    if timestamp_seconds > 0 {
        Some(timestamp_seconds)
    } else {
        None
    }
}

fn optional_timestamp_text(timestamp_seconds: u64) -> Option<String> {
    nonzero_timestamp(timestamp_seconds).map(format_utc_timestamp_secs)
}
