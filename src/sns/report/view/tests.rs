//! Module: sns::report::view::tests
//!
//! Responsibility: cover SNS report view filtering and sorting behavior.
//! Does not own: report assembly, cache IO, live source conversion, or text rendering.
//! Boundary: exercises in-memory row transformations through the view module.

use super::{proposal_matches_before, proposal_matches_status, sort_sns_proposal_rows};
use crate::sns::report::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN, SnsProposalRow, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTally, SnsProposalsSort,
};

#[test]
fn proposal_id_sort_orders_newest_id_first() {
    let mut proposals = vec![
        proposal_row(2, 100),
        proposal_row(10, 50),
        proposal_row(1, 200),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Id,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![10, 2, 1]);
}

#[test]
fn proposal_decided_sort_orders_newest_decision_first_and_open_last() {
    let mut proposals = vec![
        proposal_row_with_decision(2, Some(100)),
        proposal_row_with_decision(10, None),
        proposal_row_with_decision(1, Some(200)),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Decided,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
}

#[test]
fn proposal_status_sort_orders_lifecycle_states_first() {
    let mut proposals = vec![
        proposal_with_decision_state_and_id(2, SNS_PROPOSAL_DECISION_EXECUTED),
        proposal_with_decision_state_and_id(10, SNS_PROPOSAL_DECISION_OPEN),
        proposal_with_decision_state_and_id(1, SNS_PROPOSAL_DECISION_FAILED),
        proposal_with_decision_state_and_id(9, SNS_PROPOSAL_DECISION_DECIDED),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Status,
        SnsProposalSortDirection::Asc,
    );

    assert_eq!(proposal_ids(&proposals), vec![10, 9, 2, 1]);
}

#[test]
fn proposal_proposer_sort_orders_present_ids_before_missing_ids() {
    let mut proposals = vec![
        proposal_row_with_proposer(2, Some("ffff")),
        proposal_row_with_proposer(10, None),
        proposal_row_with_proposer(1, Some("aaaa")),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Proposer,
        SnsProposalSortDirection::Asc,
    );

    assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
}

#[test]
fn proposal_decided_ascending_sort_orders_oldest_decision_first_and_open_last() {
    let mut proposals = vec![
        proposal_row_with_decision(2, Some(100)),
        proposal_row_with_decision(10, None),
        proposal_row_with_decision(1, Some(200)),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Decided,
        SnsProposalSortDirection::Asc,
    );

    assert_eq!(proposal_ids(&proposals), vec![2, 1, 10]);
}

#[test]
fn proposal_executed_sort_orders_newest_execution_first_and_unexecuted_last() {
    let mut proposals = vec![
        proposal_row_with_execution(2, Some(100)),
        proposal_row_with_execution(10, None),
        proposal_row_with_execution(1, Some(200)),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Executed,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
}

#[test]
fn proposal_failed_sort_orders_newest_failure_first_and_non_failed_last() {
    let mut proposals = vec![
        proposal_row_with_failure(2, Some(100)),
        proposal_row_with_failure(10, None),
        proposal_row_with_failure(1, Some(200)),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Failed,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
}

#[test]
fn proposal_title_sort_orders_case_insensitive_with_id_tiebreaker() {
    let mut proposals = vec![
        proposal_row_with_title(2, "Zoo"),
        proposal_row_with_title(10, "alpha"),
        proposal_row_with_title(1, "Alpha"),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Title,
        SnsProposalSortDirection::Asc,
    );

    assert_eq!(proposal_ids(&proposals), vec![1, 10, 2]);
}

#[test]
fn proposal_action_sort_orders_descending_with_id_tiebreaker() {
    let mut proposals = vec![
        proposal_row_with_action(2, "motion"),
        proposal_row_with_action(10, "upgrade-sns-controlled-canister"),
        proposal_row_with_action(1, "motion"),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Action,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![10, 2, 1]);
}

#[test]
fn proposal_total_votes_sort_orders_highest_tally_first() {
    let mut proposals = vec![
        proposal_row_with_tally(2, Some((10, 20, 30))),
        proposal_row_with_tally(10, None),
        proposal_row_with_tally(1, Some((50, 60, 110))),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::TotalVotes,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![1, 2, 10]);
}

#[test]
fn proposal_yes_sort_orders_ascending_with_id_tiebreaker() {
    let mut proposals = vec![
        proposal_row_with_tally(2, Some((10, 20, 30))),
        proposal_row_with_tally(10, Some((10, 30, 40))),
        proposal_row_with_tally(1, Some((50, 60, 110))),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Yes,
        SnsProposalSortDirection::Asc,
    );

    assert_eq!(proposal_ids(&proposals), vec![2, 10, 1]);
}

#[test]
fn proposal_ballots_sort_orders_highest_count_first() {
    let mut proposals = vec![
        proposal_row_with_ballot_count(2, 1),
        proposal_row_with_ballot_count(10, 5),
        proposal_row_with_ballot_count(1, 5),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::Ballots,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![10, 1, 2]);
}

#[test]
fn proposal_reject_cost_sort_orders_lowest_cost_first_when_ascending() {
    let mut proposals = vec![
        proposal_row_with_reject_cost(2, 300),
        proposal_row_with_reject_cost(10, 100),
        proposal_row_with_reject_cost(1, 200),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::RejectCost,
        SnsProposalSortDirection::Asc,
    );

    assert_eq!(proposal_ids(&proposals), vec![10, 1, 2]);
}

#[test]
fn proposal_reward_round_sort_orders_highest_round_first() {
    let mut proposals = vec![
        proposal_row_with_reward_round(2, 1),
        proposal_row_with_reward_round(10, 5),
        proposal_row_with_reward_round(1, 5),
    ];

    sort_sns_proposal_rows(
        &mut proposals,
        SnsProposalsSort::RewardRound,
        SnsProposalSortDirection::Desc,
    );

    assert_eq!(proposal_ids(&proposals), vec![10, 1, 2]);
}

#[test]
fn proposal_before_filter_requires_lower_present_id() {
    assert!(proposal_matches_before(&proposal_row(9, 100), Some(10)));
    assert!(!proposal_matches_before(&proposal_row(10, 100), Some(10)));
    assert!(proposal_matches_before(&proposal_without_id(), None));
    assert!(!proposal_matches_before(&proposal_without_id(), Some(10)));
}

#[test]
fn proposal_status_filter_matches_cache_backed_statuses() {
    assert!(proposal_matches_status(
        &proposal_with_decision_state("executed"),
        SnsProposalStatusFilter::Executed
    ));
    assert!(proposal_matches_status(
        &proposal_with_decision_state("decided"),
        SnsProposalStatusFilter::Decided
    ));
    assert!(!proposal_matches_status(
        &proposal_with_decision_state("open"),
        SnsProposalStatusFilter::Failed
    ));
    assert!(!proposal_matches_status(
        &proposal_with_decision_state("adopted"),
        SnsProposalStatusFilter::Adopted
    ));
}

fn proposal_ids(proposals: &[SnsProposalRow]) -> Vec<u64> {
    proposals
        .iter()
        .filter_map(|proposal| proposal.proposal_id)
        .collect()
}

fn proposal_without_id() -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: None,
        ..proposal_row(1, 100)
    }
}

fn proposal_with_decision_state(decision_state: &str) -> SnsProposalRow {
    SnsProposalRow {
        decision_state: decision_state.to_string(),
        ..proposal_row(1, 100)
    }
}

fn proposal_with_decision_state_and_id(proposal_id: u64, decision_state: &str) -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: Some(proposal_id),
        decision_state: decision_state.to_string(),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_title(proposal_id: u64, title: &str) -> SnsProposalRow {
    SnsProposalRow {
        title: title.to_string(),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_action(proposal_id: u64, action: &str) -> SnsProposalRow {
    SnsProposalRow {
        action: action.to_string(),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_proposer(
    proposal_id: u64,
    proposer_neuron_id: Option<&str>,
) -> SnsProposalRow {
    SnsProposalRow {
        proposer_neuron_id: proposer_neuron_id.map(ToString::to_string),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_tally(proposal_id: u64, tally: Option<(u64, u64, u64)>) -> SnsProposalRow {
    SnsProposalRow {
        latest_tally: tally.map(|(yes, no, total)| SnsProposalTally {
            timestamp_seconds: 100,
            yes,
            no,
            total,
        }),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_ballot_count(proposal_id: u64, ballot_count: usize) -> SnsProposalRow {
    SnsProposalRow {
        ballot_count,
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_reject_cost(proposal_id: u64, reject_cost_e8s: u64) -> SnsProposalRow {
    SnsProposalRow {
        reject_cost_e8s,
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_reward_round(proposal_id: u64, reward_event_round: u64) -> SnsProposalRow {
    SnsProposalRow {
        reward_event_round,
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_decision(
    proposal_id: u64,
    decided_timestamp_seconds: Option<u64>,
) -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: Some(proposal_id),
        decided_timestamp_seconds,
        decided_at: decided_timestamp_seconds.map(|value| value.to_string()),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_execution(
    proposal_id: u64,
    executed_timestamp_seconds: Option<u64>,
) -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: Some(proposal_id),
        executed_timestamp_seconds,
        executed_at: executed_timestamp_seconds.map(|value| value.to_string()),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row_with_failure(
    proposal_id: u64,
    failed_timestamp_seconds: Option<u64>,
) -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: Some(proposal_id),
        failed_timestamp_seconds,
        failed_at: failed_timestamp_seconds.map(|value| value.to_string()),
        ..proposal_row(proposal_id, 100)
    }
}

fn proposal_row(proposal_id: u64, created_at_secs: u64) -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: Some(proposal_id),
        action_id: 0,
        action: "motion".to_string(),
        title: String::new(),
        summary: String::new(),
        url: None,
        decision_state: SNS_PROPOSAL_DECISION_OPEN.to_string(),
        reject_cost_e8s: 0,
        proposal_creation_timestamp_seconds: created_at_secs,
        created_at: created_at_secs.to_string(),
        decided_timestamp_seconds: None,
        decided_at: None,
        executed_timestamp_seconds: None,
        executed_at: None,
        failed_timestamp_seconds: None,
        failed_at: None,
        failure_reason: None,
        reward_event_round: 0,
        reward_event_end_timestamp_seconds: None,
        is_eligible_for_rewards: false,
        latest_tally: None,
        ballot_count: 0,
        ballots: Vec::new(),
        payload_text_rendering: None,
        proposer_neuron_id: None,
    }
}
