use super::super::{
    super::{
        SnsProposalBallotRow, SnsProposalFailureReason, SnsProposalRow, SnsProposalTally, hex_bytes,
    },
    types::{SnsGovernanceBallot, SnsGovernanceProposalData},
};
use super::common::clean_optional_text;
use crate::subnet_catalog::format_utc_timestamp_secs;

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
    }
}

fn sns_proposal_ballot_row(
    (neuron_id, ballot): (String, SnsGovernanceBallot),
) -> SnsProposalBallotRow {
    SnsProposalBallotRow {
        neuron_id,
        vote: ballot.vote,
        vote_text: ballot_vote_text(ballot.vote),
        cast_timestamp_seconds: ballot.cast_timestamp_seconds,
        cast_at: optional_timestamp_text(ballot.cast_timestamp_seconds),
        voting_power: ballot.voting_power,
    }
}

fn ballot_vote_text(vote: i32) -> String {
    match vote {
        0 => "unspecified".to_string(),
        1 => "yes".to_string(),
        2 => "no".to_string(),
        other => format!("unknown:{other}"),
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

fn proposal_action_text(action: u64) -> String {
    match action {
        0 => "unspecified".to_string(),
        1 => "motion".to_string(),
        2 => "manage_nervous_system_parameters".to_string(),
        3 => "upgrade_sns_controlled_canister".to_string(),
        4 => "add_generic_nervous_system_function".to_string(),
        5 => "remove_generic_nervous_system_function".to_string(),
        6 => "execute_generic_nervous_system_function".to_string(),
        7 => "upgrade_sns_to_next_version".to_string(),
        8 => "manage_sns_metadata".to_string(),
        9 => "transfer_sns_treasury_funds".to_string(),
        10 => "register_dapp_canisters".to_string(),
        11 => "deregister_dapp_canisters".to_string(),
        12 => "mint_sns_tokens".to_string(),
        13 => "manage_ledger_parameters".to_string(),
        14 => "manage_dapp_canister_settings".to_string(),
        15 => "advance_sns_target_version".to_string(),
        16 => "set_topics_for_custom_proposals".to_string(),
        17 => "register_extension".to_string(),
        18 => "execute_extension_operation".to_string(),
        19 => "upgrade_extension".to_string(),
        id if id >= 1_000 => format!("generic:{id}"),
        id => format!("unknown:{id}"),
    }
}

fn nonzero_timestamp(timestamp_seconds: u64) -> Option<u64> {
    (timestamp_seconds > 0).then_some(timestamp_seconds)
}

fn optional_timestamp_text(timestamp_seconds: u64) -> Option<String> {
    nonzero_timestamp(timestamp_seconds).map(format_utc_timestamp_secs)
}
