//! Module: nns::proposals::values
//!
//! Responsibility: stable argument identifiers and defaults for NNS proposal options.
//! Does not own: live NNS governance calls, report sorting, or text rendering.
//! Boundary: derives choice defaults from the report-model vocabulary.

use ic_query::nns::proposals::{
    NnsProposalListSort, NnsProposalRewardStatusFilter, NnsProposalStatusFilter,
    NnsProposalTopicFilter,
};

pub(in crate::nns::proposals) const NNS_PROPOSAL_ID_ARG: &str = "proposal-id";
pub(in crate::nns::proposals) const NNS_PROPOSAL_BALLOTS_FLAG: &str = "ballots";
pub(in crate::nns::proposals) const NNS_PROPOSAL_VERBOSE_FLAG: &str = "verbose";
pub(in crate::nns::proposals) const NNS_PROPOSAL_LIST_REWARD_STATUS_ARG: &str = "reward-status";
pub(in crate::nns) const NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL: &str =
    NnsProposalRewardStatusFilter::Any.as_str();
pub(in crate::nns) const NNS_PROPOSAL_SORT_API_LABEL: &str = NnsProposalListSort::Api.as_str();
pub(in crate::nns) const NNS_PROPOSAL_SORT_ASC_LABEL: &str = "asc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DESC_LABEL: &str = "desc";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_ANY_LABEL: &str =
    NnsProposalStatusFilter::Any.as_str();
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_ANY_LABEL: &str = NnsProposalTopicFilter::Any.as_str();
