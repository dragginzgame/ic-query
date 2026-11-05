//! Module: sns::report::model::errors
//!
//! Responsibility: typed SNS report and source errors.
//! Does not own: command usage errors, clap parsing, or text rendering.
//! Boundary: carries recoverable report-builder failures to command runners.

use crate::HostCacheError;
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// SnsHostError
///
/// Typed error returned by SNS report builders, live sources, and cache reads.
///

#[derive(Debug, ThisError)]
pub enum SnsHostError {
    #[error("Governance returned a neuron without an id")]
    MissingNeuronId,
    #[error("Governance returned an invalid neuron id")]
    InvalidNeuronId,
    #[error("SNS reports support only the mainnet `ic` network (requested {network:?})")]
    UnsupportedNetwork { network: String },

    #[error("failed to build IC agent for endpoint {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("invalid {field}: {reason}")]
    InvalidPrincipal { field: &'static str, reason: String },

    /// A source capability returned data that violates its public result contract.
    #[error("invalid {capability} source data: {reason}")]
    InvalidSourceData {
        /// Source capability whose result was rejected.
        capability: &'static str,
        /// Deterministic invariant failure.
        reason: String,
    },

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

    /// A caller supplied an SNS neuron id outside the native 32-byte contract.
    #[error(
        "invalid SNS neuron id {neuron_id}; expected exactly 64 lowercase hexadecimal characters"
    )]
    InvalidNeuronIdText {
        /// Rejected caller-supplied neuron id.
        neuron_id: String,
    },

    /// SNS Governance returned a permission entry without its required principal.
    #[error(
        "SNS governance neuron {neuron_id} permission entry {permission_index} has no principal"
    )]
    MissingNeuronPermissionPrincipal {
        /// Neuron containing the incomplete permission entry.
        neuron_id: String,
        /// Zero-based position of the incomplete permission entry.
        permission_index: usize,
    },

    #[error("failed to decode Candid response {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },

    #[error(
        "SNS neurons cache is missing at {}",
        path.display()
    )]
    MissingNeuronsCache { path: PathBuf },

    #[error("invalid SNS refresh attempt at {}: {reason}", path.display())]
    InvalidRefreshAttempt { path: PathBuf, reason: String },

    #[error("invalid SNS cache at {}: {reason}", path.display())]
    InvalidCache { path: PathBuf, reason: String },

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

    /// Shared generic JSON, filesystem, or refresh-lock cache failure.
    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error(
        "SNS neurons refresh did not publish a complete snapshot after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        pages_fetched: u32,
        rows_fetched: usize,
        reason: String,
    },

    #[error("invalid SNS refresh page size {page_size}; expected 1..={max_page_size}")]
    InvalidRefreshPageSize { page_size: u32, max_page_size: u32 },
}
