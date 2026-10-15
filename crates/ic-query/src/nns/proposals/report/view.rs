//! Module: nns::proposals::report::view
//!
//! Responsibility: apply NNS proposal list filters and ordering.
//! Does not own: proposal fetching, report assembly, command parsing, or rendering.
//! Boundary: transforms proposal rows without changing source identity.

use super::model::{
    NnsProposalListSort, NnsProposalRow, NnsProposalSortDirection, NnsProposalTopicFilter,
};
#[cfg(feature = "nns-host")]
use super::model::{NnsProposalRewardStatusFilter, NnsProposalStatusFilter};
use crate::{
    report_sort::{
        compare_optional_ord, sort_by_optional_text, sort_by_optional_u64, sort_by_text,
    },
    text_search::optional_text_contains_lowercase_query,
};

#[cfg(feature = "nns-host")]
pub(in crate::nns::proposals::report) fn proposal_matches_before(
    proposal: &NnsProposalRow,
    before_proposal_id: Option<u64>,
) -> bool {
    before_proposal_id.is_none_or(|before| {
        proposal
            .proposal_id
            .is_some_and(|proposal_id| proposal_id < before)
    })
}

#[cfg(feature = "nns-host")]
pub(in crate::nns::proposals::report) fn proposal_matches_status(
    proposal: &NnsProposalRow,
    status: NnsProposalStatusFilter,
) -> bool {
    status
        .governance_status_code()
        .is_none_or(|status_code| proposal.status == status_code)
}

#[cfg(feature = "nns-host")]
pub(in crate::nns::proposals::report) fn proposal_matches_reward_status(
    proposal: &NnsProposalRow,
    reward_status: NnsProposalRewardStatusFilter,
) -> bool {
    reward_status
        .governance_reward_status_code()
        .is_none_or(|reward_status_code| proposal.reward_status == reward_status_code)
}

pub(in crate::nns::proposals::report) fn proposal_matches_topic(
    proposal: &NnsProposalRow,
    topic: NnsProposalTopicFilter,
) -> bool {
    topic
        .topic_code()
        .is_none_or(|topic_code| proposal.topic == topic_code)
}

pub(in crate::nns::proposals::report) fn proposal_matches_proposer(
    proposal: &NnsProposalRow,
    proposer_neuron_id: Option<u64>,
) -> bool {
    proposer_neuron_id.is_none_or(|id| proposal.proposer_neuron_id == Some(id))
}

pub(in crate::nns::proposals::report) fn proposal_matches_query(
    proposal: &NnsProposalRow,
    lowercase_query: Option<&str>,
) -> bool {
    let Some(query) = lowercase_query else {
        return true;
    };
    optional_text_contains_lowercase_query(proposal.title.as_deref(), query)
        || optional_text_contains_lowercase_query(proposal.action_text.as_deref(), query)
        || optional_text_contains_lowercase_query(Some(&proposal.summary), query)
        || optional_text_contains_lowercase_query(Some(&proposal.url), query)
}

pub(in crate::nns::proposals::report) fn sort_nns_proposal_rows(
    proposals: &mut [NnsProposalRow],
    sort: NnsProposalListSort,
    direction: NnsProposalSortDirection,
) {
    let descending = matches!(direction, NnsProposalSortDirection::Desc);
    let by_id = |left: &NnsProposalRow, right: &NnsProposalRow| {
        compare_optional_ord(left.proposal_id, right.proposal_id, descending)
    };
    match sort {
        NnsProposalListSort::Api => {}
        NnsProposalListSort::Status
        | NnsProposalListSort::RewardStatus
        | NnsProposalListSort::Topic => sort_by_text(
            proposals,
            descending,
            |proposal| match sort {
                NnsProposalListSort::Status => proposal.status_text.as_str(),
                NnsProposalListSort::RewardStatus => proposal.reward_status_text.as_str(),
                NnsProposalListSort::Topic => proposal.topic_text.as_str(),
                _ => unreachable!("sort key is selected by the outer match"),
            },
            by_id,
        ),
        NnsProposalListSort::Title | NnsProposalListSort::Action => sort_by_optional_text(
            proposals,
            descending,
            |proposal| match sort {
                NnsProposalListSort::Title => proposal.title.as_deref(),
                NnsProposalListSort::Action => proposal.action_text.as_deref(),
                _ => unreachable!("sort key is selected by the outer match"),
            },
            by_id,
        ),
        _ => sort_by_optional_u64(
            proposals,
            descending,
            |proposal| match sort {
                NnsProposalListSort::Id => proposal.proposal_id,
                NnsProposalListSort::Proposer => proposal.proposer_neuron_id,
                NnsProposalListSort::Yes => proposal.latest_tally.as_ref().map(|tally| tally.yes),
                NnsProposalListSort::No => proposal.latest_tally.as_ref().map(|tally| tally.no),
                NnsProposalListSort::TotalVotes => {
                    proposal.latest_tally.as_ref().map(|tally| tally.total)
                }
                NnsProposalListSort::TallyTime => proposal
                    .latest_tally
                    .as_ref()
                    .map(|tally| tally.timestamp_seconds),
                NnsProposalListSort::VotingPower => proposal.total_potential_voting_power,
                NnsProposalListSort::Ballots => Some(proposal.ballot_count as u64),
                NnsProposalListSort::RejectCost => Some(proposal.reject_cost_e8s),
                NnsProposalListSort::RewardRound => Some(proposal.reward_event_round),
                NnsProposalListSort::Proposed => Some(proposal.proposal_timestamp_seconds),
                NnsProposalListSort::Deadline => proposal.deadline_timestamp_seconds,
                NnsProposalListSort::Decided => {
                    nonzero_timestamp(proposal.decided_timestamp_seconds)
                }
                NnsProposalListSort::Executed => {
                    nonzero_timestamp(proposal.executed_timestamp_seconds)
                }
                NnsProposalListSort::Failed => nonzero_timestamp(proposal.failed_timestamp_seconds),
                NnsProposalListSort::Api
                | NnsProposalListSort::Status
                | NnsProposalListSort::RewardStatus
                | NnsProposalListSort::Topic
                | NnsProposalListSort::Title
                | NnsProposalListSort::Action => unreachable!("nonnumeric sorts are handled above"),
            },
            by_id,
        ),
    }
}

const fn nonzero_timestamp(timestamp_seconds: u64) -> Option<u64> {
    if timestamp_seconds > 0 {
        Some(timestamp_seconds)
    } else {
        None
    }
}
