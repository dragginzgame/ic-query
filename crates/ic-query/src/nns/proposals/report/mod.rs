//! Module: nns::proposals::report
//!
//! Responsibility: build NNS governance proposal reports.
//! Does not own: CLI parsing, SNS proposal reports, cache files, or topology reports.
//! Boundary: maps live NNS governance proposal rows into text and JSON reports.

#[cfg(feature = "host")]
mod assemble;
#[cfg(feature = "host")]
mod cache;
#[cfg(feature = "host")]
mod labels;
mod model;
#[cfg(feature = "host")]
mod source;
mod text;
#[cfg(feature = "host")]
mod view;
#[cfg(feature = "host")]
mod wire;

#[cfg(feature = "host")]
use crate::{
    cache_file::CacheFileError, ic_registry::MAINNET_GOVERNANCE_CANISTER_ID, runtime::RuntimeError,
};
#[cfg(feature = "host")]
use std::{io, path::PathBuf};
#[cfg(feature = "host")]
use thiserror::Error as ThisError;

#[cfg(feature = "host")]
pub use cache::{
    DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS, NnsProposalCacheListReport,
    NnsProposalCacheListRequest, NnsProposalCacheStatusReport, NnsProposalCacheStatusRequest,
    NnsProposalCacheSummary, NnsProposalRefreshAttemptStatus, NnsProposalRefreshReport,
    NnsProposalRefreshRequest, build_nns_proposal_cache_list_report,
    build_nns_proposal_cache_status_report, build_nns_proposal_list_report_from_cache,
    build_nns_proposal_report_from_cache, nns_proposal_cache_path, nns_proposal_cache_root,
    nns_proposal_refresh_attempt_path, nns_proposal_refresh_lock_path, refresh_nns_proposal_cache,
    refresh_nns_proposal_cache_with_progress, refresh_nns_proposal_cache_with_source,
};
pub use model::{
    NnsProposalBallotRow, NnsProposalListReport, NnsProposalListRequest, NnsProposalListSort,
    NnsProposalReport, NnsProposalRequest, NnsProposalRewardStatusFilter, NnsProposalRow,
    NnsProposalSortDirection, NnsProposalStatusFilter, NnsProposalTally, NnsProposalTopicFilter,
};
#[cfg(feature = "host")]
pub use source::{
    NnsProposalSource, build_nns_proposal_list_report, build_nns_proposal_list_report_with_source,
    build_nns_proposal_report, build_nns_proposal_report_with_source,
};
#[cfg(feature = "host")]
pub use text::{
    nns_proposal_cache_list_report_text, nns_proposal_cache_status_report_text,
    nns_proposal_refresh_report_text,
};
pub use text::{nns_proposal_list_report_text, nns_proposal_report_text};

#[cfg(all(test, feature = "host"))]
mod tests;

pub const DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "host")]
/// Largest page size accepted by an NNS proposal refresh request.
pub const NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE: u32 = 100;

#[cfg(feature = "host")]
const NNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
pub(in crate::nns::proposals::report) const NNS_PROPOSAL_FETCHED_BY: &str = "ic-query";

///
/// NnsProposalHostError
///
/// Error returned while building NNS proposal reports.
///

#[derive(Debug, ThisError)]
#[cfg(feature = "host")]
pub enum NnsProposalHostError {
    #[error(
        "`icq nns proposal` supports only the mainnet `ic` network\n\nThe NNS proposal list is queried from the public Internet Computer mainnet governance canister.\nLocal replica NNS governance discovery is not supported.\n\nTry:\n  icq --network ic nns proposal list"
    )]
    LocalNetworkUnsupported,

    #[error("failed to build IC agent for {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("NNS governance agent call {method} failed: {reason}")]
    AgentCall {
        method: &'static str,
        reason: String,
    },

    #[error("failed to encode candid {message}: {reason}")]
    CandidEncode {
        message: &'static str,
        reason: String,
    },

    #[error("failed to decode candid {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },

    #[error("NNS proposal cache operation failed: {0}")]
    Cache(#[from] CacheFileError),

    #[error(
        "cached NNS proposal network mismatch: path is for {requested}, report is for {actual}"
    )]
    CacheNetworkMismatch { requested: String, actual: String },

    #[error(
        "cached NNS proposal snapshot identity mismatch at {}: {field} is {actual}, expected {expected}",
        path.display()
    )]
    CacheIdentityMismatch {
        path: PathBuf,
        field: &'static str,
        expected: String,
        actual: String,
    },

    #[error(
        "NNS proposal refresh did not publish a complete snapshot after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        pages_fetched: u32,
        rows_fetched: usize,
        reason: String,
    },

    #[error("invalid NNS proposal refresh page size {page_size}; expected 1..={max_page_size}")]
    InvalidRefreshPageSize { page_size: u32, max_page_size: u32 },

    #[error("NNS proposal {proposal_id} was not found")]
    ProposalNotFound { proposal_id: u64 },

    #[error("NNS proposals cache is missing at {}\n\nRun `icq nns proposal refresh` to fetch a complete snapshot.", path.display())]
    MissingProposalCache { path: PathBuf },

    #[error("NNS proposal list page returned a row without a proposal id")]
    MissingProposalIdInPage,

    #[error("failed to parse NNS proposal cache at {}: {source}", path.display())]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("invalid NNS proposal refresh attempt at {}: {reason}", path.display())]
    InvalidRefreshAttempt { path: PathBuf, reason: String },

    #[error("invalid NNS proposal cache at {}: {reason}", path.display())]
    InvalidCache { path: PathBuf, reason: String },

    #[error("failed to read NNS proposal cache at {}: {source}", path.display())]
    ReadCache { path: PathBuf, source: io::Error },

    #[error("failed to create Tokio runtime for NNS proposal query: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("failed to serialize NNS proposal cache JSON for {}: {source}", path.display())]
    SerializeCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported NNS proposal cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion { version: u32, expected: u32 },
}

#[cfg(feature = "host")]
fn enforce_mainnet_network(network: &str) -> Result<(), NnsProposalHostError> {
    crate::network::enforce_mainnet_network_with(network, |_| {
        NnsProposalHostError::LocalNetworkUnsupported
    })
}
