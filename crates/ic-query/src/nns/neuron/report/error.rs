//! Module: nns::neuron::report::error
//!
//! Responsibility: expose portable and native-wrapper NNS neuron failures.
//! Does not own: transport execution, cache mechanics, or process presentation.
//! Boundary: keeps canister and custom callers independent of native host dependencies.

use crate::nns::governance::NnsGovernanceError;
#[cfg(feature = "nns-host")]
use crate::{
    HostCacheError, nns::governance::NnsGovernanceAttemptReadError, runtime::RuntimeError,
};
#[cfg(feature = "nns-host")]
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// NnsNeuronError
///
/// Portable failure returned while collecting or assembling neuron reports.
///

#[derive(Debug, ThisError)]
pub enum NnsNeuronError {
    /// Shared Governance request, transport, or provenance validation failed.
    #[error(transparent)]
    Governance(#[from] NnsGovernanceError),

    /// The requested page size is outside Governance's supported range.
    #[error("invalid NNS neuron page size {page_size}; expected 1..={max_page_size}")]
    InvalidPageSize {
        /// Rejected page size.
        page_size: u32,
        /// Largest page supported by Governance.
        max_page_size: u32,
    },

    /// Governance returned its typed application-level error.
    #[error("NNS Governance rejected the neuron query with code {error_type}: {message}")]
    GovernanceResponse {
        /// Raw Governance error type.
        error_type: i32,
        /// Governance error message.
        message: String,
    },

    /// Governance has no publicly readable view for the requested neuron id.
    #[error("NNS neuron {neuron_id} was not found")]
    NeuronNotFound {
        /// Requested neuron identifier.
        neuron_id: u64,
    },

    /// Governance returned a neuron row without its required identifier.
    #[error("NNS Governance returned a neuron row without an id")]
    MissingNeuronId,

    /// A source page or detail row violated the neuron contract.
    #[error("invalid NNS neuron response: {reason}")]
    InvalidResponse {
        /// Response invariant that failed.
        reason: String,
    },
}

///
/// NnsNeuronHostError
///
/// Native wrapper failure for neuron live calls, caches, and refreshes.
///

#[cfg(feature = "nns-host")]
#[derive(Debug, ThisError)]
pub enum NnsNeuronHostError {
    /// Portable neuron collection or validation failed.
    #[error(transparent)]
    Neuron(#[from] NnsNeuronError),

    /// A capped or stalled refresh stopped before proving API exhaustion.
    #[error(
        "NNS neuron refresh stopped after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        /// Pages retained before the stop.
        pages_fetched: u32,
        /// Rows retained before the stop.
        rows_fetched: usize,
        /// Completion invariant that failed.
        reason: String,
    },

    /// A stored neuron snapshot did not match its cache key.
    #[error(
        "cached NNS neuron snapshot identity mismatch at {}: {field} is {actual}, expected {expected}",
        path.display()
    )]
    CacheIdentityMismatch {
        /// Cache path being validated.
        path: PathBuf,
        /// Identity field that did not match.
        field: &'static str,
        /// Identity required by the cache key.
        expected: String,
        /// Identity stored in the snapshot.
        actual: String,
    },

    /// A stored neuron snapshot failed family-specific validation.
    #[error("invalid NNS neuron cache at {}: {reason}", path.display())]
    InvalidCache {
        /// Cache path being validated.
        path: PathBuf,
        /// Cache invariant that failed.
        reason: String,
    },

    /// A stored refresh-attempt sidecar failed identity or lifecycle validation.
    #[error("invalid NNS neuron refresh attempt at {}: {reason}", path.display())]
    InvalidRefreshAttempt {
        /// Attempt sidecar path being validated.
        path: PathBuf,
        /// Attempt invariant that failed.
        reason: String,
    },

    /// Shared cache IO or lock handling failed.
    #[error(transparent)]
    Cache(#[from] HostCacheError),

    /// The synchronous host runtime could not execute the live query.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[cfg(feature = "nns-host")]
impl From<NnsGovernanceAttemptReadError> for NnsNeuronHostError {
    fn from(error: NnsGovernanceAttemptReadError) -> Self {
        match error {
            NnsGovernanceAttemptReadError::Cache(error) => Self::Cache(error),
            NnsGovernanceAttemptReadError::Invalid { path, reason } => {
                Self::InvalidRefreshAttempt { path, reason }
            }
        }
    }
}
