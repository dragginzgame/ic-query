//! Module: icrc::model::contracts::reports::ledger
//!
//! Responsibility: serialized ICRC ledger metadata, capability, and history contracts.
//! Does not own: account-index history, requests, live transport, archive following, or rendering.
//! Boundary: preserves raw ledger, block, archive, certificate, and capability fields.

use serde::Serialize;
use serde_json::Value as JsonValue;

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
    pub status: IcrcCapabilityStatus,
    pub details: Option<String>,
    pub error: Option<String>,
}

///
/// IcrcCapabilityStatus
///
/// Result of probing one optional ICRC ledger capability.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IcrcCapabilityStatus {
    /// The target answered the capability query successfully.
    Available,
    /// The target does not export the probed query method.
    Unsupported,
    /// The target exports the method but the query failed.
    Error,
}

impl IcrcCapabilityStatus {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_status_labels_are_stable() {
        for (status, label) in [
            (IcrcCapabilityStatus::Available, "available"),
            (IcrcCapabilityStatus::Unsupported, "unsupported"),
            (IcrcCapabilityStatus::Error, "error"),
        ] {
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{label}\"")
            );
            assert_eq!(status.as_str(), label);
        }
    }
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
