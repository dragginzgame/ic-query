//! Module: icrc::model
//!
//! Responsibility: typed ICRC command requests, reports, source rows, and errors.
//! Does not own: clap parsing, live transport, or text rendering.
//! Boundary: preserves raw ICRC fields for stable JSON output.

#[cfg(feature = "cli")]
use crate::cli::common::CurrentUnixSecsError;
#[cfg(feature = "host")]
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

impl IcrcTokenRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
        }
    }
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

impl IcrcBalanceRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        account_owner: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
            account_owner: account_owner.into(),
            subaccount_hex: None,
        }
    }

    #[must_use]
    pub fn with_subaccount_hex(mut self, subaccount_hex: impl Into<String>) -> Self {
        self.subaccount_hex = Some(subaccount_hex.into());
        self
    }
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

impl IcrcAllowanceRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        account_owner: impl Into<String>,
        spender_owner: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
            account_owner: account_owner.into(),
            account_subaccount_hex: None,
            spender_owner: spender_owner.into(),
            spender_subaccount_hex: None,
        }
    }

    #[must_use]
    pub fn with_account_subaccount_hex(
        mut self,
        account_subaccount_hex: impl Into<String>,
    ) -> Self {
        self.account_subaccount_hex = Some(account_subaccount_hex.into());
        self
    }

    #[must_use]
    pub fn with_spender_subaccount_hex(
        mut self,
        spender_subaccount_hex: impl Into<String>,
    ) -> Self {
        self.spender_subaccount_hex = Some(spender_subaccount_hex.into());
        self
    }
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

impl IcrcIndexRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
        }
    }
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

impl IcrcTransactionsRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        start: u64,
        limit: u32,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
            start,
            limit,
            follow_archives: false,
        }
    }

    #[must_use]
    pub const fn with_follow_archives(mut self, follow_archives: bool) -> Self {
        self.follow_archives = follow_archives;
        self
    }
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

impl IcrcBlockTypesRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
        }
    }
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

impl IcrcArchivesRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
            from_canister_id: None,
        }
    }

    #[must_use]
    pub fn with_from_canister_id(mut self, from_canister_id: impl Into<String>) -> Self {
        self.from_canister_id = Some(from_canister_id.into());
        self
    }
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

impl IcrcTipCertificateRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
        }
    }
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

impl IcrcCapabilitiesRequest {
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
        }
    }
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

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcBlockTypesData {
    pub block_types: Vec<IcrcBlockTypeRow>,
}

///
/// IcrcArchivesData
///
/// Source-layer ICRC-3 archive range discovery result.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcArchivesData {
    pub archives: Vec<IcrcArchiveRow>,
}

///
/// IcrcTipCertificateData
///
/// Source-layer ICRC-3 tip certificate result.
///

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcCapabilitiesData {
    pub supported_standards: Vec<IcrcTokenStandardRow>,
    pub capabilities: Vec<IcrcCapabilityRow>,
}

#[cfg(feature = "host")]
pub(in crate::icrc) fn normalize_subaccount_hex(value: &str) -> Result<String, IcrcError> {
    let bytes = subaccount_bytes_from_hex(value)?;
    Ok(hex_bytes(&bytes))
}

#[cfg(feature = "host")]
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
