//! Module: nns::governance::error
//!
//! Responsibility: expose portable and native-wrapper Governance failures.
//! Does not own: validation, transport execution, or process error presentation.
//! Boundary: preserves transport details without coupling portable callers to a runtime.

#[cfg(feature = "nns-host")]
use crate::{nns::NnsGovernanceQueryError, runtime::RuntimeError};
use thiserror::Error as ThisError;

///
/// NnsGovernanceError
///
/// Portable failure returned while collecting or assembling a Governance report.
///

#[derive(Debug, ThisError)]
pub enum NnsGovernanceError {
    /// The requested network is not the supported mainnet identity.
    #[error("direct NNS Governance reports support only the mainnet `ic` network, not {network:?}")]
    UnsupportedNetwork {
        /// Rejected network identity.
        network: String,
    },

    /// The selected transport cannot be executed by the source adapter.
    #[error("invalid NNS Governance source selection: {reason}")]
    InvalidSourceSelection {
        /// Validation failure.
        reason: String,
    },

    /// Returned provenance did not match the requested transport.
    #[error("NNS Governance source evidence does not match the request: {reason}")]
    SourceEvidenceMismatch {
        /// Evidence mismatch.
        reason: String,
    },

    /// Native IC agent setup failed.
    #[error("failed to build IC agent for {endpoint}: {reason}")]
    AgentBuild {
        /// Endpoint used to build the agent.
        endpoint: String,
        /// Agent construction failure.
        reason: String,
    },

    /// A native IC agent query failed.
    #[error("NNS Governance agent call {method} failed: {reason}")]
    AgentCall {
        /// Governance method being queried.
        method: &'static str,
        /// Query failure.
        reason: String,
    },

    /// A replicated inter-canister call could not be submitted.
    #[error("NNS Governance inter-canister call {method} failed: {reason}")]
    InterCanisterCall {
        /// Governance method being called.
        method: &'static str,
        /// Cycle-balance or call-perform failure.
        reason: String,
    },

    /// A replicated inter-canister call returned an IC reject.
    #[error(
        "NNS Governance inter-canister call {method} was rejected with code {reject_code}: {message}"
    )]
    InterCanisterCallRejected {
        /// Governance method being called.
        method: &'static str,
        /// Raw IC reject code.
        reject_code: u32,
        /// Reject message returned by the IC.
        message: String,
    },

    /// A Candid request could not be encoded.
    #[error("failed to encode Candid {message}: {reason}")]
    CandidEncode {
        /// Candid request type.
        message: &'static str,
        /// Encoding failure.
        reason: String,
    },

    /// A Candid response could not be decoded.
    #[error("failed to decode Candid {message}: {reason}")]
    CandidDecode {
        /// Candid response type.
        message: &'static str,
        /// Decoding failure.
        reason: String,
    },

    /// A response exceeded the library's explicit raw-response byte bound.
    #[error("NNS Governance {method} response was {actual_bytes} bytes; limit is {maximum_bytes}")]
    ResponseTooLarge {
        /// Governance method being decoded.
        method: &'static str,
        /// Actual response size.
        actual_bytes: usize,
        /// Maximum accepted response size.
        maximum_bytes: usize,
    },

    /// Governance returned its typed application-level error.
    #[error("NNS Governance rejected the metrics query with code {error_type}: {message}")]
    Governance {
        /// Raw Governance error type.
        error_type: i32,
        /// Governance error message.
        message: String,
    },

    /// Governance returned a non-finite metric that JSON cannot preserve.
    #[error("NNS Governance metric {field} bucket {key} has non-finite value {value}")]
    InvalidMetrics {
        /// Native Governance metric field.
        field: &'static str,
        /// Raw metric bucket key.
        key: u64,
        /// Rejected non-finite bucket value.
        value: f64,
    },
}

#[cfg(feature = "nns-host")]
impl From<NnsGovernanceQueryError> for NnsGovernanceError {
    fn from(value: NnsGovernanceQueryError) -> Self {
        match value {
            NnsGovernanceQueryError::AgentBuild { endpoint, reason } => {
                Self::AgentBuild { endpoint, reason }
            }
            NnsGovernanceQueryError::AgentCall { method, reason } => {
                Self::AgentCall { method, reason }
            }
            NnsGovernanceQueryError::CandidEncode { message, reason } => {
                Self::CandidEncode { message, reason }
            }
            NnsGovernanceQueryError::CandidDecode { message, reason } => {
                Self::CandidDecode { message, reason }
            }
        }
    }
}

///
/// NnsGovernanceHostError
///
/// Failure returned by the synchronous native Governance convenience builders.
///

#[cfg(feature = "nns-host")]
#[derive(Debug, ThisError)]
pub enum NnsGovernanceHostError {
    /// Portable collection or report validation failed.
    #[error(transparent)]
    Governance(#[from] NnsGovernanceError),

    /// The synchronous host runtime could not execute the live query.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
