//! Module: nns::proposals::report::cache
//!
//! Responsibility: complete NNS proposal snapshot refresh and inspection.
//! Does not own: command parsing, live governance transport, or text rendering.
//! Boundary: stores complete proposal snapshots and refresh-attempt metadata.

mod attempt;
mod collection;
mod model;
mod paths;
mod publish;
mod refresh;
mod reports;

pub use model::{
    NnsProposalCacheListReport, NnsProposalCacheListRequest, NnsProposalCacheStatusReport,
    NnsProposalCacheStatusRequest, NnsProposalCacheSummary, NnsProposalRefreshAttemptStatus,
    NnsProposalRefreshReport, NnsProposalRefreshRequest,
};
pub use paths::{
    nns_proposal_cache_path, nns_proposal_cache_root, nns_proposal_refresh_attempt_path,
    nns_proposal_refresh_lock_path,
};
pub use refresh::{DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS, refresh_nns_proposal_cache};
pub use reports::{
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
    build_nns_proposal_list_report_from_cache, build_nns_proposal_report_from_cache,
};

#[cfg(test)]
mod tests;

const NNS_PROPOSAL_CACHE_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
