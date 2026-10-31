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
pub enum IcrcIndexData {
    /// Index principal returned by the ledger.
    Found(String),
    /// Ledger reported that index discovery is unavailable.
    Unavailable(String),
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
