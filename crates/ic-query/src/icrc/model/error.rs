//! Module: icrc::model::error
//!
//! Responsibility: typed errors for generic ICRC parsing, reports, and live calls.
//! Does not own: command dispatch, host calls, or output policy.
//! Boundary: preserves one public error surface for reusable ICRC query behavior.

#[cfg(feature = "host")]
use crate::runtime::RuntimeError;
use thiserror::Error as ThisError;

///
/// IcrcError
///
/// Error surfaced by generic ICRC validation, report building, and live calls.
///

#[derive(Debug, ThisError)]
pub enum IcrcError {
    #[cfg(feature = "host")]
    #[error("failed to create Tokio runtime for ICRC query: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("failed to build IC agent for endpoint {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("invalid {field}: {reason}")]
    InvalidPrincipal { field: &'static str, reason: String },

    #[error("invalid subaccount hex: {reason}")]
    InvalidSubaccountHex { reason: String },

    #[error("invalid subaccount length: expected 32 bytes, got {bytes}")]
    InvalidSubaccountLength { bytes: usize },

    #[error("failed to encode Candid request for {message}: {reason}")]
    CandidEncode {
        message: &'static str,
        reason: String,
    },

    #[error("ICRC ledger method {method} failed: {reason}")]
    AgentCall {
        method: &'static str,
        reason: String,
    },

    #[error("failed to decode Candid response {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },
}
