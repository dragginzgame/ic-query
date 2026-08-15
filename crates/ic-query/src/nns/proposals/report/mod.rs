//! Module: nns::proposals::report
//!
//! Responsibility: build NNS governance proposal reports.
//! Does not own: CLI parsing, SNS proposal reports, cache files, or topology reports.
//! Boundary: maps NNS Governance proposal and collection evidence into text and JSON reports.

mod activity;
mod assemble;
#[cfg(feature = "nns-host")]
mod cache;
mod collection;
mod error;
mod model;
mod source;
mod text;
mod view;
#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32"),
    test
))]
mod wire;

pub use activity::{
    NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION, NnsProposalActivityError,
    NnsProposalActivityReport, NnsProposalActivityRequest, NnsProposalActivityValidationError,
    NnsProposalDayCount, NnsProposalRewardStatusCount, NnsProposalStatusCount,
    NnsProposalTopicCount, build_nns_proposal_activity_report,
    validate_nns_proposal_activity_report,
};
#[cfg(feature = "nns-host")]
pub use cache::{
    DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS, NnsProposalCacheListReport,
    NnsProposalCacheStatusReport, NnsProposalCacheSummary, NnsProposalRefreshReport,
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
    build_nns_proposal_list_report_from_cache, build_nns_proposal_report_from_cache,
    nns_proposal_cache_path, nns_proposal_cache_root, nns_proposal_refresh_attempt_path,
    nns_proposal_refresh_lock_path, refresh_nns_proposal_cache,
    refresh_nns_proposal_cache_with_progress, refresh_nns_proposal_cache_with_source,
};
#[cfg(feature = "nns-host")]
pub use collection::advance_nns_proposal_collection;
pub use collection::{
    NNS_PROPOSAL_COLLECTION_STATE_SCHEMA_VERSION, NnsProposalCollectionState,
    NnsProposalCollectionStatus, NnsProposalCollectionStep,
    advance_nns_proposal_collection_with_source,
};
pub use error::NnsProposalError;
#[cfg(feature = "nns-host")]
pub use error::NnsProposalHostError;
pub use model::{
    NnsProposalBallotRow, NnsProposalListReport, NnsProposalListRequest, NnsProposalListSort,
    NnsProposalReport, NnsProposalRequest, NnsProposalRewardStatus, NnsProposalRewardStatusFilter,
    NnsProposalRow, NnsProposalSortDirection, NnsProposalStatus, NnsProposalStatusFilter,
    NnsProposalTally, NnsProposalTopic, NnsProposalTopicFilter, NnsProposalVote,
};
pub use source::{
    NnsProposalSource, NnsProposalSourceFuture, build_nns_proposal_list_report_with_source,
    build_nns_proposal_report_with_source,
};
#[cfg(feature = "nns-host")]
pub use source::{build_nns_proposal_list_report, build_nns_proposal_report};
pub use text::{
    nns_proposal_activity_report_text, nns_proposal_list_report_text, nns_proposal_report_text,
};
#[cfg(feature = "nns-host")]
pub use text::{
    nns_proposal_cache_list_report_text, nns_proposal_cache_status_report_text,
    nns_proposal_refresh_report_text,
};

#[cfg(all(test, feature = "nns-host"))]
mod tests;

pub const DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT: &str = "https://icp-api.io";
/// Largest page size accepted by an NNS proposal refresh request.
pub const NNS_PROPOSAL_MAX_PAGE_SIZE: u32 = 100;
#[cfg(feature = "nns-host")]
pub const NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE: u32 = NNS_PROPOSAL_MAX_PAGE_SIZE;

const NNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
pub(in crate::nns::proposals::report) const NNS_PROPOSAL_FETCHED_BY: &str = "ic-query";

#[cfg(feature = "nns-host")]
fn enforce_mainnet_network(network: &str) -> Result<(), NnsProposalHostError> {
    crate::nns::governance::enforce_governance_mainnet_network(network)
        .map_err(NnsProposalError::from)
        .map_err(NnsProposalHostError::from)
}
