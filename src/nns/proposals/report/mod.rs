//! Module: nns::proposals::report
//!
//! Responsibility: build NNS governance proposal reports.
//! Does not own: CLI parsing, SNS proposal reports, cache files, or topology reports.
//! Boundary: maps live NNS governance proposal rows into text and JSON reports.

mod assemble;
mod cache;
mod labels;
mod model;
mod source;
mod text;
mod view;
mod wire;

use crate::{
    cache_file::CacheFileError,
    ic_registry::{DEFAULT_MAINNET_ENDPOINT, MAINNET_GOVERNANCE_CANISTER_ID},
    runtime::RuntimeError,
    subnet_catalog::MAINNET_NETWORK,
};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

pub(in crate::nns) use cache::{
    NnsProposalCacheListRequest, NnsProposalCacheStatusRequest, NnsProposalRefreshRequest,
};
pub(in crate::nns::proposals) use cache::{
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
    build_nns_proposal_list_report_from_cache, build_nns_proposal_report_from_cache,
    refresh_nns_proposal_cache,
};
pub(in crate::nns) use model::{
    NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL, NNS_PROPOSAL_SORT_API_LABEL, NNS_PROPOSAL_SORT_ASC_LABEL,
    NNS_PROPOSAL_SORT_DESC_LABEL, NNS_PROPOSAL_STATUS_ANY_LABEL, NNS_PROPOSAL_TOPIC_ANY_LABEL,
    NnsProposalListRequest, NnsProposalListSort, NnsProposalRequest, NnsProposalRewardStatusFilter,
    NnsProposalSortDirection, NnsProposalStatusFilter, NnsProposalTopicFilter,
};
pub(in crate::nns::proposals) use source::{
    build_nns_proposal_list_report, build_nns_proposal_report,
};
pub(in crate::nns::proposals) use text::{
    nns_proposal_cache_list_report_text, nns_proposal_cache_status_report_text,
    nns_proposal_list_report_text, nns_proposal_refresh_report_text, nns_proposal_report_text,
};

#[cfg(test)]
pub(in crate::nns) use model::{
    NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL, NNS_PROPOSAL_SORT_DEADLINE_LABEL,
    NNS_PROPOSAL_SORT_NONE_LABEL, NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL,
    NNS_PROPOSAL_SORT_TITLE_LABEL, NNS_PROPOSAL_SORT_VOTING_POWER_LABEL,
    NNS_PROPOSAL_STATUS_EXECUTED_LABEL, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
};

#[cfg(test)]
mod tests;

pub(in crate::nns) const DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;

const NNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::nns::proposals::report) const NNS_PROPOSAL_FETCHED_BY: &str = "ic-query";

///
/// NnsProposalHostError
///
/// Error returned while building NNS proposal reports.
///

#[derive(Debug, ThisError)]
pub enum NnsProposalHostError {
    #[error(
        "`icq nns proposal` supports only the mainnet `ic` network\n\nThe NNS proposal list is queried from the public Internet Computer mainnet governance canister.\nLocal replica NNS governance discovery is not implemented yet.\n\nTry:\n  icq --network ic nns proposal list"
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

fn enforce_mainnet_network(network: &str) -> Result<(), NnsProposalHostError> {
    if network == MAINNET_NETWORK {
        Ok(())
    } else {
        Err(NnsProposalHostError::LocalNetworkUnsupported)
    }
}
