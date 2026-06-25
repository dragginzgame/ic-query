//! Module: sns::report::model::errors
//!
//! Responsibility: typed SNS report and source errors.
//! Does not own: command usage errors, clap parsing, or text rendering.
//! Boundary: carries recoverable report-builder failures to command runners.

use crate::{cache_file::CacheFileError, runtime::RuntimeError};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// SnsHostError
///
/// Typed error returned by SNS report builders, live sources, and cache reads.
///

#[derive(Debug, ThisError)]
pub enum SnsHostError {
    #[error(
        "`icq sns` supports only the mainnet `ic` network\n\nThe SNS list is queried from the public Internet Computer mainnet SNS-W canister.\nLocal replica SNS discovery is not implemented yet.\n\nTry:\n  icq --network ic sns list"
    )]
    UnsupportedNetwork { network: String },

    #[error("failed to create Tokio runtime for SNS query: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("failed to build IC agent for endpoint {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("invalid {field}: {reason}")]
    InvalidPrincipal { field: &'static str, reason: String },

    #[error("failed to encode Candid request for {message}: {reason}")]
    CandidEncode {
        message: &'static str,
        reason: String,
    },

    #[error("SNS query method {method} failed: {reason}")]
    AgentCall {
        method: &'static str,
        reason: String,
    },

    #[error("SNS governance method {method} returned error {error_type}: {message}")]
    GovernanceError {
        method: &'static str,
        error_type: i32,
        message: String,
    },

    #[error("SNS governance method {method} returned no result")]
    MissingGovernanceResult { method: &'static str },

    #[error("failed to decode Candid response {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },

    #[error("SNS list id {id} is out of range; list contains {sns_count} deployed SNS instances")]
    UnknownSnsId { id: usize, sns_count: usize },

    #[error("could not find deployed SNS with root principal {root_canister_id}")]
    UnknownSnsRoot { root_canister_id: String },

    #[error("SNS lookup input must be a list id or root principal: {input}")]
    InvalidLookup { input: String },

    #[error(
        "SNS neurons cache is missing at {}\n\nRun `icq sns neurons refresh <id|root-principal>` to fetch a complete snapshot before using cache-backed sorting.",
        path.display()
    )]
    MissingNeuronsCache { path: PathBuf },

    #[error(
        "SNS neurons cache is missing for SNS list id {id} under {}\n\nRun `icq sns neurons refresh {id}` to fetch a complete snapshot before using cache-backed sorting.",
        root.display()
    )]
    MissingNeuronsCacheForId { id: usize, root: PathBuf },

    #[error(
        "SNS proposals cache is missing at {}\n\nRun `icq sns proposals refresh <id|root-principal>` to fetch a complete snapshot.",
        path.display()
    )]
    MissingProposalsCache { path: PathBuf },

    #[error("failed to read SNS cache at {}: {source}", path.display())]
    ReadCache { path: PathBuf, source: io::Error },

    #[error("failed to parse SNS cache at {}: {source}", path.display())]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize SNS cache JSON for {}: {source}", path.display())]
    SerializeCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported SNS cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion { version: u32, expected: u32 },

    #[error("cached SNS network mismatch: path is for {requested}, report is for {actual}")]
    CacheNetworkMismatch { requested: String, actual: String },

    #[error(
        "cached SNS snapshot identity mismatch at {}: {field} is {actual}, expected {expected}",
        path.display()
    )]
    CacheIdentityMismatch {
        path: PathBuf,
        field: &'static str,
        expected: String,
        actual: String,
    },

    #[error("SNS cache operation failed: {0}")]
    Cache(#[from] CacheFileError),

    #[error(
        "SNS neurons refresh did not publish a complete snapshot after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        pages_fetched: u32,
        rows_fetched: usize,
        reason: String,
    },

    #[error("SNS cache root is required for cache-backed neuron reports")]
    MissingCacheRoot,

    #[error("unsupported SNS proposal view: {reason}")]
    UnsupportedProposalView { reason: String },
}
