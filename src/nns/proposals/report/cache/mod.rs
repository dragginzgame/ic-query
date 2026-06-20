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

pub(in crate::nns) use model::{
    NnsProposalCacheListReport, NnsProposalCacheListRequest, NnsProposalCacheStatusReport,
    NnsProposalCacheStatusRequest, NnsProposalRefreshAttemptStatus, NnsProposalRefreshReport,
    NnsProposalRefreshRequest,
};
pub(in crate::nns::proposals) use refresh::refresh_nns_proposal_cache;
pub(in crate::nns::proposals) use reports::{
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
    build_nns_proposal_list_report_from_cache, build_nns_proposal_report_from_cache,
};

#[cfg(test)]
mod tests;

const NNS_PROPOSAL_CACHE_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
