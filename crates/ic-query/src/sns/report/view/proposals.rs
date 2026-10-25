//! Module: sns::report::view::proposals
//!
//! Responsibility: apply shared SNS proposal view filtering and ordering.
//! Does not own: proposal fetching, cache scope/limits, report assembly, or text rendering.
//! Boundary: projects already-collected proposal rows without changing cache identity.

use crate::sns::report::sns_proposal_action_label;

use crate::sns::report::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE, SnsProposalDecisionState,
    SnsProposalEligibilityFilter, SnsProposalRow, SnsProposalSortDirection,
    SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsRequest, SnsProposalsSort,
};
use crate::{
    report_sort::{
        compare_ascii_case_insensitive_text, compare_ord, sort_by_optional_text,
        sort_by_optional_u64, sort_by_text,
    },
    text_search::optional_text_contains_lowercase_query,
};

/// Apply view-only filters and ordering after live or cached scope selection.
pub(in crate::sns::report) fn apply_sns_proposal_view(
    proposals: &mut Vec<SnsProposalRow>,
    request: &SnsProposalsRequest,
) {
    let lowercase_query = request.query.as_deref().map(str::to_lowercase);
    proposals.retain(|proposal| {
        proposal_matches_eligibility(proposal, request.eligibility)
            && proposal_matches_proposer(proposal, request.proposer_neuron_id.as_deref())
            && proposal_matches_query(proposal, lowercase_query.as_deref())
    });
    sort_sns_proposal_rows(proposals, request.sort, request.sort_direction);
}

pub(in crate::sns::report) fn proposal_matches_before(
    proposal: &SnsProposalRow,
    before_proposal_id: Option<u64>,
) -> bool {
    before_proposal_id.is_none_or(|before| proposal.proposal_id < before)
}

pub(in crate::sns::report) fn proposal_matches_status(
    proposal: &SnsProposalRow,
    status: SnsProposalStatusFilter,
) -> bool {
    match status {
        SnsProposalStatusFilter::Any => true,
        SnsProposalStatusFilter::Open => proposal.decision_state == SnsProposalDecisionState::Open,
        SnsProposalStatusFilter::Decided => {
            proposal.decision_state == SnsProposalDecisionState::Decided
        }
        SnsProposalStatusFilter::Executed => {
            proposal.decision_state == SnsProposalDecisionState::Executed
        }
        SnsProposalStatusFilter::Failed => {
            proposal.decision_state == SnsProposalDecisionState::Failed
        }
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

pub(in crate::sns::report) fn proposal_matches_eligibility(
    proposal: &SnsProposalRow,
    eligibility: SnsProposalEligibilityFilter,
) -> bool {
    eligibility
        .eligibility_value()
        .is_none_or(|value| proposal.is_eligible_for_rewards == value)
}

pub(in crate::sns::report) fn proposal_matches_proposer(
    proposal: &SnsProposalRow,
    proposer_neuron_id: Option<&str>,
) -> bool {
    proposer_neuron_id.is_none_or(|proposer| {
        proposal
            .proposer_neuron_id
            .as_deref()
            .is_some_and(|neuron_id| neuron_id.starts_with(proposer))
    })
}

pub(in crate::sns::report) fn proposal_matches_query(
    proposal: &SnsProposalRow,
    lowercase_query: Option<&str>,
) -> bool {
    let Some(query) = lowercase_query else {
        return true;
    };
    let action = sns_proposal_action_label(proposal.action_id);
    optional_text_contains_lowercase_query(Some(&proposal.title), query)
        || optional_text_contains_lowercase_query(Some(&action), query)
        || optional_text_contains_lowercase_query(Some(&proposal.summary), query)
        || optional_text_contains_lowercase_query(proposal.url.as_deref(), query)
        || optional_text_contains_lowercase_query(proposal.payload_text_rendering.as_deref(), query)
}

pub(in crate::sns::report) fn sort_sns_proposal_rows(
    proposals: &mut [SnsProposalRow],
    sort: SnsProposalsSort,
    direction: SnsProposalSortDirection,
) {
    let descending = matches!(direction, SnsProposalSortDirection::Desc);
    let by_id = |left: &SnsProposalRow, right: &SnsProposalRow| {
        compare_ord(left.proposal_id, right.proposal_id, descending)
    };
    match sort {
        SnsProposalsSort::Api => {}
        SnsProposalsSort::Id => proposals.sort_by(by_id),
        SnsProposalsSort::Status => {
            sort_by_key(proposals, direction, |proposal| proposal.decision_state);
        }
        SnsProposalsSort::Action => sort_by_action(proposals, direction),
        SnsProposalsSort::Eligible => sort_by_key(proposals, direction, |proposal| {
            proposal.is_eligible_for_rewards
        }),
        SnsProposalsSort::Title => sort_by_text(
            proposals,
            descending,
            |proposal| proposal.title.as_str(),
            by_id,
        ),
        SnsProposalsSort::Topic | SnsProposalsSort::Proposer => sort_by_optional_text(
            proposals,
            descending,
            |proposal| match sort {
                SnsProposalsSort::Topic => proposal.topic.as_deref(),
                SnsProposalsSort::Proposer => proposal.proposer_neuron_id.as_deref(),
                _ => unreachable!("sort key is selected by the outer match"),
            },
            by_id,
        ),
        _ => sort_by_optional_u64(
            proposals,
            descending,
            |proposal| match sort {
                SnsProposalsSort::ActionId => Some(proposal.action_id),
                SnsProposalsSort::Yes => proposal.latest_tally.as_ref().map(|tally| tally.yes),
                SnsProposalsSort::No => proposal.latest_tally.as_ref().map(|tally| tally.no),
                SnsProposalsSort::TotalVotes => {
                    proposal.latest_tally.as_ref().map(|tally| tally.total)
                }
                SnsProposalsSort::TallyTime => proposal
                    .latest_tally
                    .as_ref()
                    .map(|tally| tally.timestamp_seconds),
                SnsProposalsSort::Ballots => Some(proposal.ballot_count as u64),
                SnsProposalsSort::RejectCost => Some(proposal.reject_cost_e8s),
                SnsProposalsSort::RewardRound => Some(proposal.reward_event_round),
                SnsProposalsSort::RewardEnd => proposal.reward_event_end_timestamp_seconds,
                SnsProposalsSort::Created => Some(proposal.proposal_creation_timestamp_seconds),
                SnsProposalsSort::Decided => proposal.decided_timestamp_seconds,
                SnsProposalsSort::Executed => proposal.executed_timestamp_seconds,
                SnsProposalsSort::Failed => proposal.failed_timestamp_seconds,
                SnsProposalsSort::Api
                | SnsProposalsSort::Id
                | SnsProposalsSort::Status
                | SnsProposalsSort::Action
                | SnsProposalsSort::Eligible
                | SnsProposalsSort::Title
                | SnsProposalsSort::Topic
                | SnsProposalsSort::Proposer => unreachable!("nonnumeric sorts are handled above"),
            },
            by_id,
        ),
    }
}

fn sort_by_action(proposals: &mut [SnsProposalRow], direction: SnsProposalSortDirection) {
    let descending = matches!(direction, SnsProposalSortDirection::Desc);
    proposals.sort_by(|left, right| {
        compare_ascii_case_insensitive_text(
            sns_proposal_action_label(left.action_id).as_ref(),
            sns_proposal_action_label(right.action_id).as_ref(),
            descending,
        )
        .then_with(|| compare_ord(left.proposal_id, right.proposal_id, descending))
    });
}

fn sort_by_key<Key: Ord>(
    proposals: &mut [SnsProposalRow],
    direction: SnsProposalSortDirection,
    key: impl Fn(&SnsProposalRow) -> Key,
) {
    let descending = matches!(direction, SnsProposalSortDirection::Desc);
    proposals.sort_by(|left, right| {
        compare_ord(key(left), key(right), descending)
            .then_with(|| compare_ord(left.proposal_id, right.proposal_id, descending))
    });
}
