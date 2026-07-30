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

///
/// IcrcAccountTransactionsError
///
/// Error surfaced while resolving and querying an ICRC account index.
///

#[derive(Debug, ThisError)]
pub enum IcrcAccountTransactionsError {
    /// The caller requested an empty page.
    #[error("invalid ICRC account transaction limit {limit}; expected at least 1")]
    InvalidLimit {
        /// Rejected page size.
        limit: u32,
    },

    /// A ledger, index, principal, Candid, or transport operation failed.
    #[error(transparent)]
    Query(#[from] IcrcError),

    /// ICRC-106 discovery could not be queried or decoded.
    #[error(
        "failed to discover an index for ledger {ledger_canister_id}; supply an explicit index canister: {source}"
    )]
    IndexDiscovery {
        /// Ledger queried for index discovery.
        ledger_canister_id: String,
        /// Underlying transport or Candid failure.
        #[source]
        source: IcrcError,
    },

    /// ICRC-106 discovery did not yield an index canister.
    #[error("ledger {ledger_canister_id} has no usable ICRC index: {reason}")]
    IndexUnavailable {
        /// Ledger queried for index discovery.
        ledger_canister_id: String,
        /// Discovery result explaining why no index is usable.
        reason: String,
    },

    /// The selected index reports a different ledger identity.
    #[error(
        "ICRC index {index_canister_id} reports ledger {actual_ledger_canister_id}, expected {expected_ledger_canister_id}"
    )]
    IndexLedgerMismatch {
        /// Index whose `ledger_id` response was checked.
        index_canister_id: String,
        /// Ledger requested by the caller.
        expected_ledger_canister_id: String,
        /// Ledger reported by the index.
        actual_ledger_canister_id: String,
    },

    /// The index returned an application-level account-history error.
    #[error("ICRC index {index_canister_id} account transaction query failed: {message}")]
    IndexQuery {
        /// Index that returned the error.
        index_canister_id: String,
        /// Index-provided error message.
        message: String,
    },
}
