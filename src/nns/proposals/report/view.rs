//! Module: nns::proposals::report::view
//!
//! Responsibility: apply NNS proposal list filters and ordering.
//! Does not own: proposal fetching, report assembly, command parsing, or rendering.
//! Boundary: transforms proposal rows without changing source identity.

use super::model::{
    NnsProposalListSort, NnsProposalRow, NnsProposalSortDirection, NnsProposalTopicFilter,
};
use std::cmp::Ordering;

pub(in crate::nns::proposals::report) fn proposal_matches_topic(
    proposal: &NnsProposalRow,
    topic: NnsProposalTopicFilter,
) -> bool {
    topic
        .topic_code()
        .is_none_or(|topic_code| proposal.topic == topic_code)
}

pub(in crate::nns::proposals::report) fn sort_nns_proposal_rows(
    proposals: &mut [NnsProposalRow],
    sort: NnsProposalListSort,
    direction: NnsProposalSortDirection,
) {
    match sort {
        NnsProposalListSort::Api => {}
        NnsProposalListSort::Id => {
            sort_by_optional_u64(proposals, direction, |proposal| proposal.proposal_id);
        }
        NnsProposalListSort::Status => {
            sort_by_text(proposals, direction, |proposal| {
                proposal.status_text.as_str()
            });
        }
        NnsProposalListSort::Topic => {
            sort_by_text(proposals, direction, |proposal| {
                proposal.topic_text.as_str()
            });
        }
        NnsProposalListSort::Proposer => {
            sort_by_optional_u64(proposals, direction, |proposal| proposal.proposer_neuron_id);
        }
        NnsProposalListSort::Title => {
            sort_by_optional_text(proposals, direction, |proposal| proposal.title.as_deref());
        }
        NnsProposalListSort::Action => {
            sort_by_optional_text(proposals, direction, |proposal| {
                proposal.action_text.as_deref()
            });
        }
        NnsProposalListSort::Yes => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                proposal.latest_tally.as_ref().map(|tally| tally.yes)
            });
        }
        NnsProposalListSort::No => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                proposal.latest_tally.as_ref().map(|tally| tally.no)
            });
        }
        NnsProposalListSort::TotalVotes => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                proposal.latest_tally.as_ref().map(|tally| tally.total)
            });
        }
        NnsProposalListSort::Ballots => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                Some(proposal.ballot_count as u64)
            });
        }
        NnsProposalListSort::RejectCost => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                Some(proposal.reject_cost_e8s)
            });
        }
        NnsProposalListSort::RewardRound => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                Some(proposal.reward_event_round)
            });
        }
        NnsProposalListSort::Proposed => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                Some(proposal.proposal_timestamp_seconds)
            });
        }
        NnsProposalListSort::Decided => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                nonzero_timestamp(proposal.decided_timestamp_seconds)
            });
        }
        NnsProposalListSort::Executed => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                nonzero_timestamp(proposal.executed_timestamp_seconds)
            });
        }
        NnsProposalListSort::Failed => {
            sort_by_optional_u64(proposals, direction, |proposal| {
                nonzero_timestamp(proposal.failed_timestamp_seconds)
            });
        }
    }
}

fn sort_by_optional_u64(
    proposals: &mut [NnsProposalRow],
    direction: NnsProposalSortDirection,
    key: impl Fn(&NnsProposalRow) -> Option<u64>,
) {
    proposals.sort_by(|left, right| {
        compare_optional_u64(key(left), key(right), direction)
            .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn sort_by_text(
    proposals: &mut [NnsProposalRow],
    direction: NnsProposalSortDirection,
    key: impl Fn(&NnsProposalRow) -> &str,
) {
    proposals.sort_by(|left, right| {
        compare_text(key(left), key(right), direction)
            .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn sort_by_optional_text(
    proposals: &mut [NnsProposalRow],
    direction: NnsProposalSortDirection,
    key: impl for<'a> Fn(&'a NnsProposalRow) -> Option<&'a str>,
) {
    proposals.sort_by(|left, right| {
        compare_optional_text(key(left), key(right), direction)
            .then_with(|| compare_optional_u64(left.proposal_id, right.proposal_id, direction))
    });
}

fn compare_optional_text(
    left: Option<&str>,
    right: Option<&str>,
    direction: NnsProposalSortDirection,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_text(left, right, direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_text(left: &str, right: &str, direction: NnsProposalSortDirection) -> Ordering {
    let left_key = left.to_ascii_lowercase();
    let right_key = right.to_ascii_lowercase();
    match direction {
        NnsProposalSortDirection::Asc => left_key.cmp(&right_key),
        NnsProposalSortDirection::Desc => right_key.cmp(&left_key),
    }
}

fn compare_optional_u64(
    left: Option<u64>,
    right: Option<u64>,
    direction: NnsProposalSortDirection,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_ord(left, right, direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_ord<T>(left: T, right: T, direction: NnsProposalSortDirection) -> Ordering
where
    T: Ord,
{
    match direction {
        NnsProposalSortDirection::Asc => left.cmp(&right),
        NnsProposalSortDirection::Desc => right.cmp(&left),
    }
}

const fn nonzero_timestamp(timestamp_seconds: u64) -> Option<u64> {
    if timestamp_seconds > 0 {
        Some(timestamp_seconds)
    } else {
        None
    }
}
