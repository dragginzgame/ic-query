//! Module: icrc::model::data
//!
//! Responsibility: source-layer data returned by ICRC source adapters.
//! Does not own: public report envelopes, request construction, errors, or rendering.
//! Boundary: carries raw source results into report assembly without display conversion.

use super::IcrcTokenStandardRow;
use super::contracts::{
    IcrcAccountTransactionRow, IcrcArchiveFollowErrorRow, IcrcArchivedBlocksRow, IcrcCapabilityRow,
    IcrcFollowedArchiveBlockRow, IcrcTransactionBlockRow,
};
use std::path::PathBuf;

///
/// IcrcBalanceData
///
/// Source-layer balance result plus enough token metadata for display.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcBalanceData {
    pub token_symbol: String,
    pub decimals: u8,
    pub balance: String,
}

///
/// IcrcAllowanceData
///
/// Source-layer allowance result plus enough token metadata for display.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAllowanceData {
    pub token_symbol: String,
    pub decimals: u8,
    pub allowance: String,
    pub expires_at_unix_nanos: Option<String>,
}

///
/// IcrcAccountTransactionPageData
///
/// Source-layer account-history page returned by an ICRC index canister.
///

#[derive(Clone, Debug, Eq, PartialEq)]
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
/// IcrcAccountTransactionCollectionData
///
/// Complete account history returned after a source exhausts one verified index.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAccountTransactionCollectionData {
    /// Verified index canister used for every page.
    pub index_canister_id: String,
    /// Balance returned by the first page.
    pub balance: String,
    /// Ledger token symbol.
    pub token_symbol: String,
    /// Ledger token decimals.
    pub decimals: u8,
    /// Canonical newest-first transaction rows.
    pub transactions: Vec<IcrcAccountTransactionRow>,
    /// Number of index pages fetched.
    pub page_count: u32,
    /// Last cursor observed while exhausting the index.
    pub last_cursor: Option<String>,
}

///
/// CachedIcrcAccountTransactionSnapshot
///
/// Validated complete account-history snapshot paired with its local path.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedIcrcAccountTransactionSnapshot {
    /// Local cache path.
    pub path: PathBuf,
    /// Validated complete snapshot.
    pub snapshot: super::contracts::IcrcAccountTransactionSnapshot,
}

///
/// IcrcIndexData
///
/// Source-layer ICRC-106 index discovery result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcIndexData {
    pub index_canister_id: Option<String>,
    pub index_error: Option<String>,
}

///
/// IcrcTransactionsData
///
/// Source-layer ICRC-3 block history result from a ledger canister.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcTransactionsData {
    /// Total number of blocks as a decimal natural, or unknown for custom sources.
    /// Returned blocks and archive ranges must stay below this length when known.
    pub log_length: Option<String>,
    pub blocks: Vec<IcrcTransactionBlockRow>,
    pub archived_blocks: Vec<IcrcArchivedBlocksRow>,
    pub followed_archive_blocks: Vec<IcrcFollowedArchiveBlockRow>,
    pub archive_follow_errors: Vec<IcrcArchiveFollowErrorRow>,
}

///
/// IcrcCapabilitiesData
///
/// Source-layer generic ICRC ledger capability probe result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcCapabilitiesData {
    pub supported_standards: Vec<IcrcTokenStandardRow>,
    pub capabilities: Vec<IcrcCapabilityRow>,
}
