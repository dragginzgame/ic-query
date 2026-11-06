//! Module: icrc::model::reports::ledger
//!
//! Responsibility: serialized ICRC ledger metadata, capability, and history contracts.
//! Does not own: account-index history, requests, live transport, archive following, or rendering.
//! Boundary: preserves raw ledger, block, archive, certificate, and capability fields.

use crate::icrc::{IcrcIndexData, IcrcTokenData, IcrcTokenStandardRow};
use serde::Serialize;
use serde_json::Value as JsonValue;

///
/// IcrcTokenReport
///
/// Serializable report for generic ICRC ledger token metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTokenReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    /// Raw ledger metadata shared with token source capabilities.
    #[serde(flatten)]
    pub token: IcrcTokenData,
}

///
/// IcrcIndexReport
///
/// Serializable report for one generic ICRC-106 index discovery lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcIndexReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    /// Discovered index principal or the source's unavailability reason.
    pub index: IcrcIndexData,
}

///
/// IcrcTransactionsData
///
/// Source-layer ICRC-3 block history result from a ledger canister.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTransactionsData {
    /// Total number of blocks as a decimal natural, or unknown for custom sources.
    /// Returned blocks and archive ranges must stay below this length when known.
    pub log_length: Option<String>,
    /// Blocks returned directly by the ledger.
    pub blocks: Vec<IcrcTransactionBlockRow>,
    /// Archive referrals returned for the requested page.
    pub archived_blocks: Vec<IcrcArchivedBlocksRow>,
    /// Blocks retained from successful archive callbacks.
    pub followed_archive_blocks: Vec<IcrcFollowedArchiveBlockRow>,
    /// Bounded failures retained while following archive callbacks.
    pub archive_follow_errors: Vec<IcrcArchiveFollowErrorRow>,
}

///
/// IcrcTransactionsReport
///
/// Serializable report for a generic ICRC ledger transaction/block history page.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTransactionsReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    pub requested_start: String,
    pub requested_limit: u32,
    pub follow_archives: bool,
    /// Canonical history payload.
    #[serde(flatten)]
    pub transactions: IcrcTransactionsData,
}

///
/// IcrcBlockTypesReport
///
/// Serializable report for generic ICRC-3 supported block type discovery.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcBlockTypesReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    pub block_types: Vec<IcrcBlockTypeRow>,
}

///
/// IcrcArchivesReport
///
/// Serializable report for generic ICRC-3 archive range discovery.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcArchivesReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    pub from_canister_id: Option<String>,
    pub archives: Vec<IcrcArchiveRow>,
}

///
/// IcrcTipCertificateReport
///
/// Serializable report for a generic ICRC-3 ledger tip certificate.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTipCertificateReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    /// Complete certificate/witness evidence, absent when the ledger returns none.
    pub certificate: Option<IcrcTipCertificateEvidence>,
}

///
/// IcrcTipCertificateEvidence
///
/// Original certificate and hash-tree reply evidence with their byte lengths.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTipCertificateEvidence {
    /// Original certificate bytes as lowercase hexadecimal text.
    pub certificate_hex: String,
    /// Number of bytes in the original certificate.
    pub certificate_bytes: usize,
    /// Original hash-tree bytes as lowercase hexadecimal text.
    pub hash_tree_hex: String,
    /// Number of bytes in the original hash tree.
    pub hash_tree_bytes: usize,
}

///
/// IcrcCapabilitiesReport
///
/// Serializable report for generic ICRC ledger endpoint capabilities.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcCapabilitiesReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
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
    /// Canonical block summary and raw evidence.
    #[serde(flatten)]
    pub block: IcrcTransactionBlockRow,
}

#[cfg(test)]
mod tests;

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
