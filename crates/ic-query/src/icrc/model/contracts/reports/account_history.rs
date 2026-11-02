//! Module: icrc::model::contracts::reports::account_history
//!
//! Responsibility: serialized ICRC account-history page, cache, and row contracts.
//! Does not own: ledger-wide history, requests, live transport, cache mechanics, or rendering.
//! Boundary: owns the stable JSON and persisted snapshot shapes for indexed account history.

use crate::cache::{CacheCollectionCompleteness, CacheValidationStatus};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use serde_json::Value as JsonValue;

///
/// IcrcAccountTransactionPageData
///
/// Source-layer account-history page returned by an ICRC index canister.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionPageData {
    /// Index canister that answered the query.
    pub index_canister_id: String,
    /// Account balance reported by the index.
    pub balance: String,
    /// Oldest transaction id known for the account.
    pub oldest_transaction_id: Option<String>,
    /// Cursor for requesting the next older page.
    pub next_start: Option<String>,
    /// Ledger token symbol.
    pub token_symbol: String,
    /// Ledger token decimals.
    pub decimals: u8,
    /// Account transactions in index response order.
    pub transactions: Vec<IcrcAccountTransactionRow>,
}

///
/// IcrcAccountTransactionPageReport
///
/// Serializable report for a backward page of ICRC index account transactions.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionPageReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    /// Queried account owner principal.
    pub account_owner: String,
    /// Queried subaccount as normalized hex.
    pub subaccount_hex: Option<String>,
    /// Exclusive block-index cursor supplied by the caller.
    pub requested_start: Option<String>,
    /// Maximum number of transactions requested.
    pub requested_limit: u32,
    /// Validated index response and ledger display metadata.
    #[serde(flatten)]
    pub page: IcrcAccountTransactionPageData,
}

///
/// IcrcAccountTransactionContext
///
/// Collected account identity and acquisition metadata shared by complete-history views.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcrcAccountTransactionContext {
    /// Ledger canister whose history was collected.
    pub ledger_canister_id: String,
    /// Verified index canister used for every page.
    pub index_canister_id: String,
    /// Queried account owner principal.
    pub account_owner: String,
    /// Queried subaccount as normalized hex.
    pub subaccount_hex: Option<String>,
    /// Explicit IC API endpoint used for collection.
    pub source_endpoint: String,
    /// Complete collection start timestamp.
    pub collection_started_at: String,
    /// Complete collection finish timestamp.
    pub collection_completed_at: String,
    /// Collector identity.
    pub fetched_by: String,
}

///
/// IcrcAccountTransactionSnapshot
///
/// Complete persisted account-history snapshot collected by exhausting the index API.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IcrcAccountTransactionSnapshot {
    /// Shared identity and collection metadata, flattened in JSON.
    #[serde(flatten)]
    pub context: IcrcAccountTransactionContext,
    /// Cache schema version.
    pub schema_version: u32,
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
    pub completeness: CacheCollectionCompleteness,
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
    /// Shared identity and collection metadata, flattened in JSON.
    #[serde(flatten)]
    pub context: IcrcAccountTransactionContext,
    /// Report schema version.
    pub schema_version: u32,
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
    /// Shared identity and collection metadata, flattened in JSON.
    #[serde(flatten)]
    pub context: IcrcAccountTransactionContext,
    /// Report schema version.
    pub schema_version: u32,
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
/// Existing-file admission outcome; invalid files carry no invented collection metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "cache_status", rename_all = "snake_case")]
pub enum IcrcAccountTransactionCacheSummary {
    /// One admitted complete snapshot.
    Valid {
        /// Verified collection facts, flattened in JSON.
        #[serde(flatten)]
        data: IcrcAccountTransactionValidCacheSummary,
    },
    /// Existing file that failed admission.
    Invalid {
        /// Existing file path.
        cache_path: String,
        /// Original validation failure.
        cache_error: String,
    },
}

impl IcrcAccountTransactionCacheSummary {
    /// Return the typed admission status.
    #[must_use]
    pub const fn cache_status(&self) -> CacheValidationStatus {
        match self {
            Self::Valid { .. } => CacheValidationStatus::Valid,
            Self::Invalid { .. } => CacheValidationStatus::Invalid,
        }
    }
}

///
/// IcrcAccountTransactionValidCacheSummary
///
/// Facts read only from an admitted complete account-history snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAccountTransactionValidCacheSummary {
    /// Verified index canister.
    pub index_canister_id: String,
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
    /// Explicit or resolved index canister recorded by the attempt.
    pub index_canister_id: Option<String>,
    /// Validated shared lifecycle evidence.
    #[serde(flatten)]
    pub lifecycle: crate::cache::SnapshotRefreshAttemptStatus,
}

///
/// IcrcAccountRow
///
/// Structured ledger account or legacy ICP account identifier in transaction rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(tag = "account_kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum IcrcAccountRow {
    /// ICRC ledger account identity.
    Structured {
        /// Account owner principal.
        owner: String,
        /// Optional 32-byte subaccount as lowercase hex.
        subaccount_hex: Option<String>,
    },
    /// Legacy ICP account identifier retained exactly as returned by the index.
    Legacy {
        /// Legacy account identifier text.
        account_identifier: String,
    },
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

#[cfg(test)]
mod account_tests {
    use super::IcrcAccountRow;
    use serde_json::json;

    #[test]
    fn account_forms_round_trip_and_refuse_ambiguous_or_former_shapes() {
        for account in [
            IcrcAccountRow::Structured {
                owner: "aaaaa-aa".to_string(),
                subaccount_hex: None,
            },
            IcrcAccountRow::Legacy {
                account_identifier: "original-index-identifier".to_string(),
            },
        ] {
            let value = serde_json::to_value(&account).unwrap();
            assert_eq!(
                serde_json::from_value::<IcrcAccountRow>(value).unwrap(),
                account
            );
        }
        for value in [
            json!({"owner":"aaaaa-aa", "subaccount_hex":null, "account_identifier":null}),
            json!({"account_kind":"legacy", "account_identifier":"id", "owner":"aaaaa-aa"}),
            json!({"account_kind":"structured", "subaccount_hex":null}),
            json!({"account_kind":"structured", "owner":"aaaaa-aa", "account_identifier":"id"}),
        ] {
            assert!(serde_json::from_value::<IcrcAccountRow>(value).is_err());
        }
    }
}
