//! Module: nns::proposals::report::view
//!
//! Responsibility: apply NNS proposal list filters and ordering.
//! Does not own: proposal fetching, report assembly, command parsing, or rendering.
//! Boundary: transforms proposal rows without changing source identity.

use super::model::{
    NnsProposalListRequest, NnsProposalListSort, NnsProposalRewardStatus, NnsProposalRow,
    NnsProposalSortDirection, NnsProposalStatus, NnsProposalTopic, NnsProposalTopicFilter,
};
#[cfg(feature = "nns-host")]
use super::model::{NnsProposalRewardStatusFilter, NnsProposalStatusFilter};
use crate::{
    report_sort::{ReportSortKey, compare_optional_ord},
    text_search::optional_text_contains_lowercase_query,
};

/// Apply only the local view filters and ordering shared by live and cached lists.
pub(in crate::nns::proposals::report) fn project_nns_proposal_rows(
    proposals: impl IntoIterator<Item = NnsProposalRow>,
    request: &NnsProposalListRequest,
) -> Vec<NnsProposalRow> {
    let lowercase_query = request.query.as_deref().map(str::to_lowercase);
    let mut proposals = proposals
        .into_iter()
        .filter(|proposal| proposal_matches_proposer(proposal, request.proposer_neuron_id))
        .filter(|proposal| proposal_matches_topic(proposal, request.topic))
        .filter(|proposal| proposal_matches_query(proposal, lowercase_query.as_deref()))
        .collect::<Vec<_>>();
    sort_nns_proposal_rows(&mut proposals, request.sort, request.sort_direction);
    proposals
}

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
    let key: ReportSortKey<NnsProposalRow> = match sort {
        NnsProposalListSort::Api => return,
        NnsProposalListSort::Status => ReportSortKey::Text(|proposal: &NnsProposalRow| {
            NnsProposalStatus::from_code(proposal.status).as_str()
        }),
        NnsProposalListSort::RewardStatus => ReportSortKey::Text(|proposal| {
            NnsProposalRewardStatus::from_code(proposal.reward_status).as_str()
        }),
        NnsProposalListSort::Topic => {
            ReportSortKey::Text(|proposal| NnsProposalTopic::from_code(proposal.topic).as_str())
        }
        NnsProposalListSort::Title => {
            ReportSortKey::OptionalText(|proposal| proposal.title.as_deref())
        }
        NnsProposalListSort::Action => {
            ReportSortKey::OptionalText(|proposal| proposal.action_text.as_deref())
        }
        NnsProposalListSort::Id => ReportSortKey::OptionalU64(|proposal| proposal.proposal_id),
        NnsProposalListSort::Proposer => {
            ReportSortKey::OptionalU64(|proposal| proposal.proposer_neuron_id)
        }
        NnsProposalListSort::Yes => ReportSortKey::OptionalU64(|proposal| {
            proposal.latest_tally.as_ref().map(|tally| tally.yes)
        }),
        NnsProposalListSort::No => ReportSortKey::OptionalU64(|proposal| {
            proposal.latest_tally.as_ref().map(|tally| tally.no)
        }),
        NnsProposalListSort::TotalVotes => ReportSortKey::OptionalU64(|proposal| {
            proposal.latest_tally.as_ref().map(|tally| tally.total)
        }),
        NnsProposalListSort::TallyTime => ReportSortKey::OptionalU64(|proposal| {
            proposal
                .latest_tally
                .as_ref()
                .map(|tally| tally.timestamp_seconds)
        }),
        NnsProposalListSort::VotingPower => {
            ReportSortKey::OptionalU64(|proposal| proposal.total_potential_voting_power)
        }
        NnsProposalListSort::Ballots => {
            ReportSortKey::OptionalU64(|proposal| Some(proposal.ballot_count as u64))
        }
        NnsProposalListSort::RejectCost => {
            ReportSortKey::OptionalU64(|proposal| Some(proposal.reject_cost_e8s))
        }
        NnsProposalListSort::RewardRound => {
            ReportSortKey::OptionalU64(|proposal| Some(proposal.reward_event_round))
        }
        NnsProposalListSort::Proposed => {
            ReportSortKey::OptionalU64(|proposal| Some(proposal.proposal_timestamp_seconds))
        }
        NnsProposalListSort::Deadline => {
            ReportSortKey::OptionalU64(|proposal| proposal.deadline_timestamp_seconds)
        }
        NnsProposalListSort::Decided => ReportSortKey::OptionalU64(|proposal| {
            nonzero_timestamp(proposal.decided_timestamp_seconds)
        }),
        NnsProposalListSort::Executed => ReportSortKey::OptionalU64(|proposal| {
            nonzero_timestamp(proposal.executed_timestamp_seconds)
        }),
        NnsProposalListSort::Failed => ReportSortKey::OptionalU64(|proposal| {
            nonzero_timestamp(proposal.failed_timestamp_seconds)
        }),
    };
    key.sort(proposals, descending, |left, right| {
        compare_optional_ord(left.proposal_id, right.proposal_id, descending)
    });
}

const fn nonzero_timestamp(timestamp_seconds: u64) -> Option<u64> {
    if timestamp_seconds > 0 {
        Some(timestamp_seconds)
    } else {
        None
    }
}
