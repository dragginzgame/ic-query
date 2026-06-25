//! Module: icrc::model
//!
//! Responsibility: typed ICRC command requests, reports, source rows, and errors.
//! Does not own: clap parsing, live transport, or text rendering.
//! Boundary: preserves raw ICRC fields for stable JSON output.

#[cfg(feature = "cli")]
use crate::cli::common::CurrentUnixSecsError;
use crate::hex::hex_bytes;
#[cfg(feature = "host")]
use crate::runtime::RuntimeError;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::io;
use thiserror::Error as ThisError;

///
/// IcrcError
///
/// Error surfaced by generic ICRC command parsing, report building, and live calls.
///
#[derive(Debug, ThisError)]
pub enum IcrcError {
    #[error("{0}")]
    Usage(String),

    #[cfg(feature = "cli")]
    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),

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

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

///
/// IcrcTokenRequest
///
/// Request accepted by the generic ICRC token metadata report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcTokenRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
}

///
/// IcrcBalanceRequest
///
/// Request accepted by the generic ICRC account balance report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcBalanceRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
    pub account_owner: String,
    pub subaccount_hex: Option<String>,
}

///
/// IcrcAllowanceRequest
///
/// Request accepted by the generic ICRC allowance report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAllowanceRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
    pub account_owner: String,
    pub account_subaccount_hex: Option<String>,
    pub spender_owner: String,
    pub spender_subaccount_hex: Option<String>,
}

///
/// IcrcIndexRequest
///
/// Request accepted by the generic ICRC index discovery report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcIndexRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
}

///
/// IcrcTransactionsRequest
///
/// Request accepted by the generic ICRC transaction history report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcTransactionsRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
    pub start: u64,
    pub limit: u32,
    pub follow_archives: bool,
}

///
/// IcrcBlockTypesRequest
///
/// Request accepted by the generic ICRC supported block types report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcBlockTypesRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
}

///
/// IcrcArchivesRequest
///
/// Request accepted by the generic ICRC archives report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcArchivesRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
    pub from_canister_id: Option<String>,
}

///
/// IcrcTipCertificateRequest
///
/// Request accepted by the generic ICRC-3 tip certificate report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcTipCertificateRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
}

///
/// IcrcCapabilitiesRequest
///
/// Request accepted by the generic ICRC ledger capabilities report builder.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcCapabilitiesRequest {
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub ledger_canister_id: String,
}

///
/// IcrcTokenReport
///
/// Serializable report for generic ICRC ledger token metadata.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTokenReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
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
/// IcrcBalanceReport
///
/// Serializable report for one generic ICRC account balance lookup.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcBalanceReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub account_owner: String,
    pub subaccount_hex: Option<String>,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub balance: String,
}

///
/// IcrcAllowanceReport
///
/// Serializable report for one generic ICRC allowance lookup.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAllowanceReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub account_owner: String,
    pub account_subaccount_hex: Option<String>,
    pub spender_owner: String,
    pub spender_subaccount_hex: Option<String>,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub allowance: String,
    pub expires_at_unix_nanos: Option<String>,
}

///
/// IcrcIndexReport
///
/// Serializable report for one generic ICRC-106 index discovery lookup.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcIndexReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub index_canister_id: Option<String>,
    pub index_error: Option<String>,
}

///
/// IcrcTransactionsReport
///
/// Serializable report for a generic ICRC ledger transaction/block history page.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTransactionsReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub requested_start: String,
    pub requested_limit: u32,
    pub follow_archives: bool,
    pub log_length: Option<String>,
    pub blocks: Vec<IcrcTransactionBlockRow>,
    pub archived_blocks: Vec<IcrcArchivedBlocksRow>,
    pub followed_archive_blocks: Vec<IcrcFollowedArchiveBlockRow>,
    pub archive_follow_errors: Vec<IcrcArchiveFollowErrorRow>,
}

///
/// IcrcBlockTypesReport
///
/// Serializable report for generic ICRC-3 supported block type discovery.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcBlockTypesReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub block_types: Vec<IcrcBlockTypeRow>,
}

///
/// IcrcArchivesReport
///
/// Serializable report for generic ICRC-3 archive range discovery.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcArchivesReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub from_canister_id: Option<String>,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub archives: Vec<IcrcArchiveRow>,
}

///
/// IcrcTipCertificateReport
///
/// Serializable report for a generic ICRC-3 ledger tip certificate.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTipCertificateReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub certificate_present: bool,
    pub certificate_hex: Option<String>,
    pub certificate_bytes: Option<usize>,
    pub hash_tree_hex: Option<String>,
    pub hash_tree_bytes: Option<usize>,
}

///
/// IcrcCapabilitiesReport
///
/// Serializable report for generic ICRC ledger endpoint capabilities.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcCapabilitiesReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub supported_standards: Vec<IcrcTokenStandardRow>,
    pub capabilities: Vec<IcrcCapabilityRow>,
}

///
/// IcrcCapabilityRow
///
/// Serializable row for one probed generic ICRC ledger capability.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcCapabilityRow {
    pub capability: String,
    pub method: String,
    pub status: String,
    pub details: Option<String>,
    pub error: Option<String>,
}

///
/// IcrcTokenStandardRow
///
/// Serializable row for one ICRC standard supported by a ledger.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTokenStandardRow {
    pub name: String,
    pub url: String,
}

///
/// IcrcTokenMetadataRow
///
/// Serializable row for one raw ICRC ledger metadata entry.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTokenMetadataRow {
    pub key: String,
    pub value_type: String,
    pub value: JsonValue,
}

///
/// IcrcTransactionBlockRow
///
/// Serializable row for one ICRC-3 block returned by a ledger canister.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTransactionBlockRow {
    pub index: String,
    pub block_type: Option<String>,
    pub transaction_kind: Option<String>,
    pub timestamp_unix_nanos: Option<String>,
    pub amount_base_units: Option<String>,
    pub raw_block: JsonValue,
}

///
/// IcrcArchivedBlocksRow
///
/// Serializable row for one ICRC-3 archive callback returned by a ledger canister.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcArchivedBlocksRow {
    pub callback_canister_id: String,
    pub callback_method: String,
    pub ranges: Vec<IcrcArchivedRangeRow>,
}

///
/// IcrcArchivedRangeRow
///
/// Serializable row for one ICRC-3 archived block range.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcArchivedRangeRow {
    pub start: String,
    pub length: String,
}

///
/// IcrcFollowedArchiveBlockRow
///
/// Serializable row for one ICRC-3 block fetched from an archive callback.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcFollowedArchiveBlockRow {
    pub archive_canister_id: String,
    pub callback_method: String,
    pub index: String,
    pub block_type: Option<String>,
    pub transaction_kind: Option<String>,
    pub timestamp_unix_nanos: Option<String>,
    pub amount_base_units: Option<String>,
    pub raw_block: JsonValue,
}

///
/// IcrcArchiveFollowErrorRow
///
/// Serializable row for one archive callback that could not be followed.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcArchiveFollowErrorRow {
    pub callback_canister_id: String,
    pub callback_method: String,
    pub ranges: Vec<IcrcArchivedRangeRow>,
    pub error: String,
}

///
/// IcrcBlockTypeRow
///
/// Serializable row for one supported ICRC-3 block type.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcBlockTypeRow {
    pub block_type: String,
    pub url: String,
}

///
/// IcrcArchiveRow
///
/// Serializable row for one ICRC-3 archive range.
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcArchiveRow {
    pub canister_id: String,
    pub start: String,
    pub end: String,
}

///
/// IcrcTokenData
///
/// Source-layer token metadata returned by an ICRC ledger.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTokenData {
    pub(in crate::icrc) token_name: String,
    pub(in crate::icrc) token_symbol: String,
    pub(in crate::icrc) decimals: u8,
    pub(in crate::icrc) transfer_fee: String,
    pub(in crate::icrc) total_supply: String,
    pub(in crate::icrc) minting_account_owner: Option<String>,
    pub(in crate::icrc) minting_account_subaccount_hex: Option<String>,
    pub(in crate::icrc) supported_standards: Vec<IcrcTokenStandardRow>,
    pub(in crate::icrc) metadata: Vec<IcrcTokenMetadataRow>,
}

///
/// IcrcBalanceData
///
/// Source-layer balance result plus enough token metadata for display.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcBalanceData {
    pub(in crate::icrc) token_symbol: String,
    pub(in crate::icrc) decimals: u8,
    pub(in crate::icrc) balance: String,
}

///
/// IcrcAllowanceData
///
/// Source-layer allowance result plus enough token metadata for display.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcAllowanceData {
    pub(in crate::icrc) token_symbol: String,
    pub(in crate::icrc) decimals: u8,
    pub(in crate::icrc) allowance: String,
    pub(in crate::icrc) expires_at_unix_nanos: Option<String>,
}

///
/// IcrcIndexData
///
/// Source-layer ICRC-106 index discovery result.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcIndexData {
    pub(in crate::icrc) index_canister_id: Option<String>,
    pub(in crate::icrc) index_error: Option<String>,
}

///
/// IcrcTransactionsData
///
/// Source-layer ICRC-3 block history result from a ledger canister.
///
#[derive(Clone, Debug, PartialEq)]
pub(in crate::icrc) struct IcrcTransactionsData {
    pub(in crate::icrc) log_length: Option<String>,
    pub(in crate::icrc) blocks: Vec<IcrcTransactionBlockRow>,
    pub(in crate::icrc) archived_blocks: Vec<IcrcArchivedBlocksRow>,
    pub(in crate::icrc) followed_archive_blocks: Vec<IcrcFollowedArchiveBlockRow>,
    pub(in crate::icrc) archive_follow_errors: Vec<IcrcArchiveFollowErrorRow>,
}

///
/// IcrcBlockTypesData
///
/// Source-layer ICRC-3 supported block types result.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcBlockTypesData {
    pub(in crate::icrc) block_types: Vec<IcrcBlockTypeRow>,
}

///
/// IcrcArchivesData
///
/// Source-layer ICRC-3 archive range discovery result.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcArchivesData {
    pub(in crate::icrc) archives: Vec<IcrcArchiveRow>,
}

///
/// IcrcTipCertificateData
///
/// Source-layer ICRC-3 tip certificate result.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcTipCertificateData {
    pub(in crate::icrc) certificate_hex: Option<String>,
    pub(in crate::icrc) certificate_bytes: Option<usize>,
    pub(in crate::icrc) hash_tree_hex: Option<String>,
    pub(in crate::icrc) hash_tree_bytes: Option<usize>,
}

///
/// IcrcCapabilitiesData
///
/// Source-layer generic ICRC ledger capability probe result.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::icrc) struct IcrcCapabilitiesData {
    pub(in crate::icrc) supported_standards: Vec<IcrcTokenStandardRow>,
    pub(in crate::icrc) capabilities: Vec<IcrcCapabilityRow>,
}

pub(in crate::icrc) fn normalize_subaccount_hex(value: &str) -> Result<String, IcrcError> {
    let bytes = subaccount_bytes_from_hex(value)?;
    Ok(hex_bytes(&bytes))
}

pub(in crate::icrc) fn subaccount_bytes_from_hex(value: &str) -> Result<Vec<u8>, IcrcError> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        return Err(IcrcError::InvalidSubaccountHex {
            reason: "hex string must contain an even number of characters".to_string(),
        });
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16).map_err(|err| {
                IcrcError::InvalidSubaccountHex {
                    reason: err.to_string(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if bytes.len() != 32 {
        return Err(IcrcError::InvalidSubaccountLength { bytes: bytes.len() });
    }
    Ok(bytes)
}
