//! Module: sns::report::view::proposals
//!
//! Responsibility: apply SNS proposal list view ordering.
//! Does not own: proposal fetching, cache filtering, report assembly, or text rendering.
//! Boundary: sorts proposal rows without changing cache identity.

use crate::sns::report::{
    SNS_PROPOSAL_DECISION_DECIDED, SNS_PROPOSAL_DECISION_EXECUTED, SNS_PROPOSAL_DECISION_FAILED,
    SNS_PROPOSAL_DECISION_OPEN, SNS_PROPOSAL_STATUS_ADOPTED_CODE,
    SNS_PROPOSAL_STATUS_REJECTED_CODE, SnsProposalRow, SnsProposalSortDirection,
    SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort,
};
use std::cmp::Ordering;

pub(in crate::sns::report) fn proposal_matches_before(
    proposal: &SnsProposalRow,
    before_proposal_id: Option<u64>,
) -> bool {
    before_proposal_id.is_none_or(|before| {
        proposal
            .proposal_id
            .is_some_and(|proposal_id| proposal_id < before)
    })
}

pub(in crate::sns::report) fn proposal_matches_status(
    proposal: &SnsProposalRow,
    status: SnsProposalStatusFilter,
) -> bool {
    match status {
        SnsProposalStatusFilter::Any => true,
        SnsProposalStatusFilter::Open => proposal.decision_state == SNS_PROPOSAL_DECISION_OPEN,
        SnsProposalStatusFilter::Decided => {
            proposal.decision_state == SNS_PROPOSAL_DECISION_DECIDED
        }
        SnsProposalStatusFilter::Executed => {
            proposal.decision_state == SNS_PROPOSAL_DECISION_EXECUTED
        }
        SnsProposalStatusFilter::Failed => proposal.decision_state == SNS_PROPOSAL_DECISION_FAILED,
        SnsProposalStatusFilter::Rejected => {
            proposal.status == Some(SNS_PROPOSAL_STATUS_REJECTED_CODE)
        }
        SnsProposalStatusFilter::Adopted => {
            proposal.status == Some(SNS_PROPOSAL_STATUS_ADOPTED_CODE)
        }
    }
}

pub(in crate::sns::report) fn proposal_matches_topic(
    proposal: &SnsProposalRow,
    topic: SnsProposalTopicFilter,
) -> bool {
    topic
        .topic_label()
        .is_none_or(|topic_label| proposal.topic.as_deref() == Some(topic_label))
}

pub(in crate::sns::report) fn sort_sns_proposal_rows(
    proposals: &mut [SnsProposalRow],
    sort: SnsProposalsSort,
    direction: SnsProposalSortDirection,
) {
    match sort {
        SnsProposalsSort::Api => {}
        SnsProposalsSort::Id => {
            sort_by_optional_u64(proposals, direction, |proposal| proposal.proposal_id);
        }
        SnsProposalsSort::Status => {
            sort_by_status(proposals, direction);
        }
        SnsProposalsSort::Topic => {
            sort_by_optional_text(proposals, direction, |proposal| proposal.topic.as_deref());
        }
        SnsProposalsSort::Proposer => {
            sort_by_optional_text(proposals, direction, |proposal| {
                proposal.proposer_neuron_id.as_deref()
            });
        }
        SnsProposalsSort::Title => {
            sort_by_text(proposals, direction, |proposal| proposal.title.as_str());
        }
        SnsProposalsSort::Action => {
            sort_by_text(proposals, direction, |proposal| proposal.action.as_str());
        }
        SnsProposalsSort::Yes => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.latest_tally.as_ref().map(|tally| tally.yes)
        }),
        SnsProposalsSort::No => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.latest_tally.as_ref().map(|tally| tally.no)
        }),
        SnsProposalsSort::TotalVotes => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.latest_tally.as_ref().map(|tally| tally.total)
        }),
        SnsProposalsSort::Ballots => sort_by_optional_u64(proposals, direction, |proposal| {
            Some(proposal.ballot_count as u64)
        }),
        SnsProposalsSort::RejectCost => sort_by_optional_u64(proposals, direction, |proposal| {
            Some(proposal.reject_cost_e8s)
        }),
        SnsProposalsSort::RewardRound => sort_by_optional_u64(proposals, direction, |proposal| {
            Some(proposal.reward_event_round)
        }),
        SnsProposalsSort::Created => sort_by_optional_u64(proposals, direction, |proposal| {
            Some(proposal.proposal_creation_timestamp_seconds)
        }),
        SnsProposalsSort::Decided => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.decided_timestamp_seconds
        }),
        SnsProposalsSort::Executed => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.executed_timestamp_seconds
        }),
        SnsProposalsSort::Failed => sort_by_optional_u64(proposals, direction, |proposal| {
            proposal.failed_timestamp_seconds
        }),
    }
}

fn sort_by_optional_u64(
    proposals: &mut [SnsProposalRow],
    direction: SnsProposalSortDirection,
    key: impl Fn(&SnsProposalRow) -> Option<u64>,
) {
    proposals.sort_by(|left, right| {
        compare_optional_u64(key(left), key(right), direction)
            .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn sort_by_text(
    proposals: &mut [SnsProposalRow],
    direction: SnsProposalSortDirection,
    key: impl Fn(&SnsProposalRow) -> &str,
) {
    proposals.sort_by(|left, right| {
        compare_text(key(left), key(right), direction)
            .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn sort_by_optional_text(
    proposals: &mut [SnsProposalRow],
    direction: SnsProposalSortDirection,
    key: impl for<'a> Fn(&'a SnsProposalRow) -> Option<&'a str>,
) {
    proposals.sort_by(|left, right| {
        compare_optional_text(key(left), key(right), direction)
            .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn sort_by_status(proposals: &mut [SnsProposalRow], direction: SnsProposalSortDirection) {
    proposals.sort_by(|left, right| {
        compare_ord(
            status_sort_rank(&left.decision_state),
            status_sort_rank(&right.decision_state),
            direction,
        )
        .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn status_sort_rank(decision_state: &str) -> u8 {
    match decision_state {
        state if state == SNS_PROPOSAL_DECISION_OPEN => 0,
        state if state == SNS_PROPOSAL_DECISION_DECIDED => 1,
        state if state == SNS_PROPOSAL_DECISION_EXECUTED => 2,
        state if state == SNS_PROPOSAL_DECISION_FAILED => 3,
        _ => 4,
    }
}

fn compare_optional_text(
    left: Option<&str>,
    right: Option<&str>,
    direction: SnsProposalSortDirection,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_text(left, right, direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_text(left: &str, right: &str, direction: SnsProposalSortDirection) -> Ordering {
    let left_key = left.to_ascii_lowercase();
    let right_key = right.to_ascii_lowercase();
    match direction {
        SnsProposalSortDirection::Asc => left_key.cmp(&right_key),
        SnsProposalSortDirection::Desc => right_key.cmp(&left_key),
    }
}

fn compare_optional_u64(
    left: Option<u64>,
    right: Option<u64>,
    direction: SnsProposalSortDirection,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_ord(left, right, direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_ord<T>(left: T, right: T, direction: SnsProposalSortDirection) -> Ordering
where
    T: Ord,
{
    match direction {
        SnsProposalSortDirection::Asc => left.cmp(&right),
        SnsProposalSortDirection::Desc => right.cmp(&left),
    }
}
