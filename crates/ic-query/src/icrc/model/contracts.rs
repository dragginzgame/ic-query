//! Module: icrc::model::contracts
//!
//! Responsibility: public ICRC request, report, and serializable row contracts.
//! Does not own: errors, source-layer data, subaccount validation, live transport, or rendering.
//! Boundary: preserves the public request API and raw JSON report fields.

use serde::Serialize;
use serde_json::Value as JsonValue;

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
