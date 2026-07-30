//! Module: nns::proposals
//!
//! Responsibility: expose NNS governance proposal request and report APIs.
//! Does not own: SNS proposal queries, registry inventory, or topology reports.
//! Boundary: groups reusable proposal models, builders, caches, sources, and renderers.

mod report;

#[cfg(feature = "host")]
pub use report::{
    DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS, NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE,
    NnsProposalCacheListReport, NnsProposalCacheStatusReport, NnsProposalCacheSummary,
    NnsProposalHostError, NnsProposalRefreshReport, NnsProposalSource,
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
    build_nns_proposal_list_report, build_nns_proposal_list_report_from_cache,
    build_nns_proposal_list_report_with_source, build_nns_proposal_report,
    build_nns_proposal_report_from_cache, build_nns_proposal_report_with_source,
    nns_proposal_cache_list_report_text, nns_proposal_cache_path, nns_proposal_cache_root,
    nns_proposal_cache_status_report_text, nns_proposal_refresh_attempt_path,
    nns_proposal_refresh_lock_path, nns_proposal_refresh_report_text, refresh_nns_proposal_cache,
    refresh_nns_proposal_cache_with_progress, refresh_nns_proposal_cache_with_source,
};
pub use report::{
    DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT, NnsProposalBallotRow, NnsProposalListReport,
    NnsProposalListRequest, NnsProposalListSort, NnsProposalReport, NnsProposalRequest,
    NnsProposalRewardStatusFilter, NnsProposalRow, NnsProposalSortDirection,
    NnsProposalStatusFilter, NnsProposalTally, NnsProposalTopicFilter,
    nns_proposal_list_report_text, nns_proposal_report_text,
};
