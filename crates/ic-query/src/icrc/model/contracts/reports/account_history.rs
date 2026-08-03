//! Module: icrc::model::contracts::reports::account_history
//!
//! Responsibility: serialized ICRC account-history page, cache, and row contracts.
//! Does not own: ledger-wide history, requests, live transport, cache mechanics, or rendering.
//! Boundary: owns the stable JSON and persisted snapshot shapes for indexed account history.

use crate::cache::CacheValidationStatus;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use serde_json::Value as JsonValue;

///
/// IcrcAccountTransactionPageReport
///
/// Serializable report for a backward page of ICRC index account transactions.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionPageReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Ledger canister whose transactions were indexed.
    pub ledger_canister_id: String,
    /// Index canister that answered the account-history query.
    pub index_canister_id: String,
    /// Queried account owner principal.
    pub account_owner: String,
    /// Queried subaccount as normalized hex.
    pub subaccount_hex: Option<String>,
    /// Exclusive block-index cursor supplied by the caller.
    pub requested_start: Option<String>,
    /// Maximum number of transactions requested.
    pub requested_limit: u32,
    /// Cursor to pass as `start` to request the next older page.
    pub next_start: Option<String>,
    /// Oldest transaction id known for this account.
    pub oldest_transaction_id: Option<String>,
    /// Account balance reported by the index at its synchronized tip.
    pub balance: String,
    /// Ledger token symbol used for text rendering.
    pub token_symbol: String,
    /// Ledger token decimals used for text rendering.
    pub decimals: u8,
    /// Collection timestamp in UTC text form.
    pub fetched_at: String,
    /// IC API endpoint used for ledger and index calls.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Transactions returned by the index in its native page order.
    pub transactions: Vec<IcrcAccountTransactionRow>,
}

///
/// IcrcAccountTransactionCompleteness
///
/// Evidence that a persisted account-history snapshot exhausted the index API.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcrcAccountTransactionCompleteness {
    /// Stable completeness classification; complete snapshots use `api_exhausted`.
    pub status: String,
    /// Maximum transactions requested per source page.
    pub page_size: u32,
    /// Number of source pages collected.
    pub page_count: u32,
    /// Number of unique persisted transaction rows.
    pub row_count: usize,
    /// Whether the source guarantees every page belongs to one point in time.
    pub point_in_time_guaranteed: bool,
}

///
/// IcrcAccountTransactionSnapshot
///
/// Complete persisted account-history snapshot collected by exhausting the index API.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcrcAccountTransactionSnapshot {
    /// Cache schema version.
    pub schema_version: u32,
    /// IC API endpoint used for ledger and index calls.
    pub source_endpoint: String,
    /// Collection start timestamp.
    pub collection_started_at: String,
    /// Collection completion timestamp.
    pub collection_completed_at: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Ledger canister whose transactions were indexed.
    pub ledger_canister_id: String,
    /// Verified index canister used for every page.
    pub index_canister_id: String,
    /// Queried account owner principal.
    pub account_owner: String,
    /// Queried subaccount as normalized hex.
    pub subaccount_hex: Option<String>,
    /// Account balance reported by the first index page.
    pub balance: String,
    /// Ledger token symbol used for text rendering.
    pub token_symbol: String,
    /// Ledger token decimals used for text rendering.
    pub decimals: u8,
    /// Highest collected transaction id.
    pub newest_transaction_id: Option<String>,
    /// Lowest collected transaction id.
    pub oldest_transaction_id: Option<String>,
    /// Complete-collection evidence.
    pub completeness: IcrcAccountTransactionCompleteness,
    /// Canonical newest-first account transactions.
    pub transactions: Vec<IcrcAccountTransactionRow>,
}

///
/// IcrcAccountTransactionRefreshReport
///
/// Serializable forced-refresh outcome for one complete account-history cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionRefreshReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Ledger canister whose account history was collected.
    pub ledger_canister_id: String,
    /// Verified index canister used for every page.
    pub index_canister_id: String,
    /// Queried account owner principal.
    pub account_owner: String,
    /// Queried subaccount as normalized hex.
    pub subaccount_hex: Option<String>,
    /// Number of unique transactions published.
    pub transaction_count: usize,
    /// Highest published transaction id.
    pub newest_transaction_id: Option<String>,
    /// Lowest published transaction id.
    pub oldest_transaction_id: Option<String>,
    /// Maximum transactions requested per source page.
    pub page_size: u32,
    /// Number of source pages collected.
    pub page_count: u32,
    /// Whether the source guarantees one point-in-time snapshot.
    pub point_in_time_guaranteed: bool,
    /// Whether a prior complete cache existed.
    pub replaced_existing_cache: bool,
    /// Non-fatal error encountered finalizing the refresh-attempt sidecar.
    pub attempt_finalization_error: Option<String>,
    /// Collection start timestamp.
    pub collection_started_at: String,
    /// Collection completion timestamp.
    pub collection_completed_at: String,
    /// IC API endpoint used for ledger and index calls.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Published complete-cache path.
    pub cache_path: String,
    /// Refresh-attempt sidecar path.
    pub refresh_attempt_path: String,
    /// Refresh lock path.
    pub refresh_lock_path: String,
}

///
/// IcrcAccountTransactionListReport
///
/// Serializable cache-only view over a complete account-history snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionListReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Ledger canister whose cached history is shown.
    pub ledger_canister_id: String,
    /// Verified index canister used to collect the cache.
    pub index_canister_id: String,
    /// Cached account owner principal.
    pub account_owner: String,
    /// Cached subaccount as normalized hex.
    pub subaccount_hex: Option<String>,
    /// Maximum cached rows requested by this view.
    pub requested_limit: u32,
    /// Stable requested ordering name.
    pub sort: String,
    /// Total rows in the complete cache.
    pub total_transaction_count: usize,
    /// Rows returned by this view.
    pub returned_transaction_count: usize,
    /// Highest transaction id in the complete cache.
    pub newest_transaction_id: Option<String>,
    /// Lowest transaction id in the complete cache.
    pub oldest_transaction_id: Option<String>,
    /// Account balance captured from the first index page.
    pub balance: String,
    /// Ledger token symbol used for text rendering.
    pub token_symbol: String,
    /// Ledger token decimals used for text rendering.
    pub decimals: u8,
    /// Complete collection start timestamp.
    pub collection_started_at: String,
    /// Complete collection finish timestamp.
    pub collection_completed_at: String,
    /// IC API endpoint represented by the cache.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Whether source exhaustion was proven.
    pub complete: bool,
    /// Whether the source guaranteed one point-in-time snapshot.
    pub point_in_time_guaranteed: bool,
    /// Maximum transactions requested per source page.
    pub page_size: u32,
    /// Number of source pages collected.
    pub page_count: u32,
    /// Complete-cache path read by this view.
    pub cache_path: String,
    /// Selected cached rows in requested order.
    pub transactions: Vec<IcrcAccountTransactionRow>,
}

///
/// IcrcAccountTransactionCacheStatusReport
///
/// Serializable local cache and latest-refresh status.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionCacheStatusReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Ledger canister in the requested cache identity.
    pub ledger_canister_id: String,
    /// Account owner in the requested cache identity.
    pub account_owner: String,
    /// Subaccount in the requested cache identity.
    pub subaccount_hex: Option<String>,
    /// IC API endpoint in the requested cache identity.
    pub source_endpoint: String,
    /// Whether a cache file exists at the expected path.
    pub found: bool,
    /// Validation summary when a cache file exists.
    pub cache: Option<IcrcAccountTransactionCacheSummary>,
    /// Expected complete-cache path.
    pub expected_cache_path: String,
    /// Refresh-attempt sidecar path.
    pub refresh_attempt_path: String,
    /// Refresh lock path.
    pub refresh_lock_path: String,
    /// Latest refresh-attempt state when present.
    pub latest_attempt: Option<IcrcAccountTransactionRefreshAttemptStatus>,
}

///
/// IcrcAccountTransactionCacheSummary
///
/// Serializable validation summary for one complete account-history cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionCacheSummary {
    /// Stable cache validation status.
    pub cache_status: CacheValidationStatus,
    /// Validation error when the existing cache is invalid.
    pub cache_error: Option<String>,
    /// Verified index canister when the cache is valid.
    pub index_canister_id: Option<String>,
    /// Number of cached transaction rows.
    pub transaction_count: usize,
    /// Highest cached transaction id.
    pub newest_transaction_id: Option<String>,
    /// Lowest cached transaction id.
    pub oldest_transaction_id: Option<String>,
    /// Maximum transactions requested per source page.
    pub page_size: u32,
    /// Number of source pages collected.
    pub page_count: u32,
    /// Whether source exhaustion was proven.
    pub complete: bool,
    /// Whether the source guaranteed one point-in-time snapshot.
    pub point_in_time_guaranteed: bool,
    /// Complete collection start timestamp.
    pub collection_started_at: String,
    /// Complete collection finish timestamp.
    pub collection_completed_at: String,
    /// Complete-cache path.
    pub cache_path: String,
}

///
/// IcrcAccountTransactionRefreshAttemptStatus
///
/// Serializable status of the latest complete-history refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionRefreshAttemptStatus {
    /// Stable attempt lifecycle status.
    pub status: String,
    /// Attempt start timestamp.
    pub started_at: String,
    /// Last attempt update timestamp.
    pub updated_at: String,
    /// Explicit or resolved index canister recorded by the attempt.
    pub index_canister_id: Option<String>,
    /// Maximum transactions requested per source page.
    pub page_size: u32,
    /// Successfully collected pages.
    pub pages_fetched: u32,
    /// Rows retained before the latest update.
    pub rows_fetched: usize,
    /// Last exclusive cursor when present.
    pub last_cursor: Option<String>,
    /// Final failure text when the attempt failed.
    pub last_error: Option<String>,
}

///
/// IcrcAccountRow
///
/// Serializable ICRC account identity used in account-transaction rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcrcAccountRow {
    /// ICRC account owner principal when the index uses structured accounts.
    pub owner: Option<String>,
    /// Optional 32-byte subaccount as lowercase hex.
    pub subaccount_hex: Option<String>,
    /// Legacy ICP account identifier when the index returns identifier text.
    pub account_identifier: Option<String>,
}

///
/// IcrcAccountTransactionRow
///
/// Serializable projected and lossless JSON representation of one index transaction.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcrcAccountTransactionRow {
    /// Ledger block index of the transaction.
    pub id: String,
    /// Index-reported transaction kind.
    pub kind: String,
    /// Ledger transaction timestamp as Unix nanoseconds when present.
    pub timestamp_unix_nanos: Option<String>,
    /// Operation amount in ledger base units when the operation carries one.
    pub amount_base_units: Option<String>,
    /// Operation fee in ledger base units when the operation carries one.
    pub fee_base_units: Option<String>,
    /// Source account when present.
    pub from: Option<IcrcAccountRow>,
    /// Destination account when present.
    pub to: Option<IcrcAccountRow>,
    /// Spender account when present.
    pub spender: Option<IcrcAccountRow>,
    /// Operation memo as lowercase hex when present.
    pub memo_hex: Option<String>,
    /// Caller-supplied creation time as Unix nanoseconds when present.
    pub created_at_time_unix_nanos: Option<String>,
    /// Approval expiry as Unix nanoseconds when present.
    pub expires_at_unix_nanos: Option<String>,
    /// Expected prior allowance in base units when present.
    pub expected_allowance_base_units: Option<String>,
    /// Lossless JSON projection of every typed transaction field returned by the index.
    pub raw_transaction: JsonValue,
}
