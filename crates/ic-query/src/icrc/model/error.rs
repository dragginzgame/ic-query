//! Module: icrc::model::error
//!
//! Responsibility: typed errors for generic ICRC parsing, reports, and live calls.
//! Does not own: command dispatch, host calls, or output policy.
//! Boundary: preserves one public error surface for reusable ICRC query behavior.

#[cfg(feature = "host")]
use crate::{HostCacheError, runtime::RuntimeError};
use std::path::PathBuf;
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
/// IcrcAccountTransactionError
///
/// Error surfaced while resolving and querying an ICRC account index.
///

#[derive(Debug, ThisError)]
pub enum IcrcAccountTransactionError {
    /// A cache identity omitted its endpoint.
    #[error("invalid ICRC account transaction source endpoint {value:?}: {reason}")]
    InvalidSourceEndpoint {
        /// Rejected endpoint.
        value: String,
        /// Validation failure.
        reason: String,
    },

    /// A page or refresh requested an unsupported page size.
    #[error(
        "invalid ICRC account transaction page size {page_size}; expected between 1 and {max_page_size}"
    )]
    InvalidPageSize {
        /// Rejected page size.
        page_size: u32,
        /// Largest supported page size.
        max_page_size: u32,
    },

    /// A cache view requested no rows.
    #[error("invalid ICRC account transaction list limit {limit}; expected at least 1")]
    InvalidListLimit {
        /// Rejected view limit.
        limit: u32,
    },

    /// A diagnostic refresh bound cannot prove any collection progress.
    #[error("invalid ICRC account transaction max pages {max_pages}; expected at least 1")]
    InvalidMaxPages {
        /// Rejected page bound.
        max_pages: u32,
    },

    /// A caller supplied a non-decimal or otherwise invalid candid Nat cursor.
    #[error("invalid ICRC account transaction cursor {value:?}: {reason}")]
    InvalidCursor {
        /// Rejected cursor text.
        value: String,
        /// Validation failure.
        reason: String,
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

    /// Complete collection stopped before the source API was exhausted.
    #[error(
        "incomplete ICRC account transaction collection after {pages_fetched} page(s) and {rows_fetched} row(s): {reason}"
    )]
    IncompleteCollection {
        /// Successfully fetched pages.
        pages_fetched: u32,
        /// Unique rows retained.
        rows_fetched: usize,
        /// Last exclusive cursor when present.
        last_cursor: Option<String>,
        /// Reason the collection could not be proven complete.
        reason: String,
    },

    /// A page fetch failed after collection had begun.
    #[error(
        "ICRC account transaction collection failed after {pages_fetched} page(s) and {rows_fetched} row(s): {source}"
    )]
    CollectionPage {
        /// Successfully fetched pages before the failure.
        pages_fetched: u32,
        /// Unique rows retained before the failure.
        rows_fetched: usize,
        /// Last exclusive cursor when present.
        last_cursor: Option<String>,
        /// Underlying typed page failure.
        #[source]
        source: Box<Self>,
    },

    /// A complete cache failed semantic validation.
    #[error("invalid ICRC account transaction cache at {}: {reason}", path.display())]
    InvalidCache {
        /// Cache path.
        path: PathBuf,
        /// Validation failure.
        reason: String,
    },

    /// A refresh-attempt sidecar failed semantic validation.
    #[error(
        "invalid ICRC account transaction refresh attempt at {}: {reason}",
        path.display()
    )]
    InvalidRefreshAttempt {
        /// Attempt sidecar path.
        path: PathBuf,
        /// Validation failure.
        reason: String,
    },

    /// A cache load, lock, or atomic-write operation failed.
    #[cfg(feature = "host")]
    #[error(transparent)]
    Cache(#[from] HostCacheError),
}
