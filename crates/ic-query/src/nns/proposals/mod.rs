//! Module: nns::proposals
//!
//! Responsibility: NNS governance proposal commands.
//! Does not own: SNS proposal queries, registry inventory, or topology reports.
//! Boundary: parses NNS proposal commands and renders live governance reports.

#[cfg(feature = "cli")]
mod commands;
#[cfg(feature = "cli")]
mod options;
pub mod report;
#[cfg(feature = "cli")]
mod run;
#[cfg(feature = "cli")]
mod values;

#[cfg(feature = "host")]
pub use report::{
    DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS, LiveNnsProposalSource,
    NnsProposalCacheListReport, NnsProposalCacheListRequest, NnsProposalCacheStatusReport,
    NnsProposalCacheStatusRequest, NnsProposalCacheSummary, NnsProposalHostError,
    NnsProposalRefreshAttemptStatus, NnsProposalRefreshReport, NnsProposalRefreshRequest,
    NnsProposalSource, NnsProposalSourceRequest, build_nns_proposal_cache_list_report,
    build_nns_proposal_cache_status_report, build_nns_proposal_list_report,
    build_nns_proposal_list_report_from_cache, build_nns_proposal_list_report_with_source,
    build_nns_proposal_report, build_nns_proposal_report_from_cache,
    build_nns_proposal_report_with_source, nns_proposal_cache_list_report_text,
    nns_proposal_cache_path, nns_proposal_cache_root, nns_proposal_cache_status_report_text,
    nns_proposal_refresh_attempt_path, nns_proposal_refresh_lock_path,
    nns_proposal_refresh_report_text, refresh_nns_proposal_cache,
    refresh_nns_proposal_cache_with_source,
};
pub use report::{
    DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT, NnsProposalBallotRow, NnsProposalListReport,
    NnsProposalListRequest, NnsProposalListSort, NnsProposalReport, NnsProposalRequest,
    NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalSortDirection,
    NnsProposalStatusFilter, NnsProposalTally, NnsProposalTopicFilter,
    nns_proposal_list_report_text, nns_proposal_report_text,
};

#[cfg(feature = "cli")]
pub(in crate::nns) use run::run;

#[cfg(all(test, feature = "cli"))]
pub(in crate::nns) use commands::{
    nns_proposal_cache_list_usage, nns_proposal_cache_status_usage, nns_proposal_cache_usage,
    nns_proposal_info_usage, nns_proposal_list_usage, nns_proposal_refresh_usage,
    nns_proposal_usage,
};
#[cfg(all(test, feature = "cli"))]
pub(in crate::nns) use options::{
    NnsProposalCacheOptions, NnsProposalListOptions, NnsProposalOptions, NnsProposalRefreshOptions,
};
#[cfg(all(test, feature = "cli"))]
pub(in crate::nns) use report::{
    NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL, NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL,
    NNS_PROPOSAL_SORT_API_LABEL, NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_DEADLINE_LABEL,
    NNS_PROPOSAL_SORT_NONE_LABEL, NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL,
    NNS_PROPOSAL_SORT_TALLY_TIME_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
    NNS_PROPOSAL_SORT_VOTING_POWER_LABEL, NNS_PROPOSAL_STATUS_ANY_LABEL,
    NNS_PROPOSAL_STATUS_EXECUTED_LABEL, NNS_PROPOSAL_TOPIC_ANY_LABEL,
    NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
};
