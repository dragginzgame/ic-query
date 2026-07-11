//! Module: icrc::model::data
//!
//! Responsibility: source-layer data returned by ICRC source adapters.
//! Does not own: public report envelopes, request construction, errors, or rendering.
//! Boundary: carries raw source results into report assembly without display conversion.

use super::contracts::{
    IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcBlockTypeRow,
    IcrcCapabilityRow, IcrcFollowedArchiveBlockRow, IcrcTokenMetadataRow, IcrcTokenStandardRow,
    IcrcTransactionBlockRow,
};

///
/// IcrcTokenData
///
/// Source-layer token metadata returned by an ICRC ledger.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcTokenData {
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub transfer_fee: String,
    pub total_supply: String,
    pub minting_account_owner: Option<String>,
    pub minting_account_subaccount_hex: Option<String>,
    pub supported_standards: Vec<IcrcTokenStandardRow>,
    pub metadata: Vec<IcrcTokenMetadataRow>,
}

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
    pub log_length: Option<String>,
    pub blocks: Vec<IcrcTransactionBlockRow>,
    pub archived_blocks: Vec<IcrcArchivedBlocksRow>,
    pub followed_archive_blocks: Vec<IcrcFollowedArchiveBlockRow>,
    pub archive_follow_errors: Vec<IcrcArchiveFollowErrorRow>,
}

///
/// IcrcBlockTypesData
///
/// Source-layer ICRC-3 supported block types result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcBlockTypesData {
    pub block_types: Vec<IcrcBlockTypeRow>,
}

///
/// IcrcArchivesData
///
/// Source-layer ICRC-3 archive range discovery result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcArchivesData {
    pub archives: Vec<IcrcArchiveRow>,
}

///
/// IcrcTipCertificateData
///
/// Source-layer ICRC-3 tip certificate result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcTipCertificateData {
    pub certificate_hex: Option<String>,
    pub certificate_bytes: Option<usize>,
    pub hash_tree_hex: Option<String>,
    pub hash_tree_bytes: Option<usize>,
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
