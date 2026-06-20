//! Module: nns::proposals
//!
//! Responsibility: NNS governance proposal commands.
//! Does not own: SNS proposal queries, registry inventory, or topology reports.
//! Boundary: parses NNS proposal commands and renders live governance reports.

mod commands;
mod options;
mod report;
mod run;
mod values;

pub(in crate::nns) use report::NnsProposalHostError;
pub(in crate::nns) use run::run;

#[cfg(test)]
pub(in crate::nns) use commands::{
    nns_proposal_cache_list_usage, nns_proposal_cache_status_usage, nns_proposal_cache_usage,
    nns_proposal_info_usage, nns_proposal_list_usage, nns_proposal_refresh_usage,
    nns_proposal_usage,
};
#[cfg(test)]
pub(in crate::nns) use options::{
    NnsProposalCacheOptions, NnsProposalListOptions, NnsProposalOptions, NnsProposalRefreshOptions,
};
#[cfg(test)]
pub(in crate::nns) use report::{
    DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT, NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL,
    NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL, NNS_PROPOSAL_SORT_API_LABEL,
    NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_NONE_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
    NNS_PROPOSAL_STATUS_ANY_LABEL, NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
    NNS_PROPOSAL_TOPIC_ANY_LABEL, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL, NnsProposalListSort,
    NnsProposalRewardStatusFilter, NnsProposalSortDirection, NnsProposalStatusFilter,
    NnsProposalTopicFilter,
};
