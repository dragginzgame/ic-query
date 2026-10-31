//! Canonical token metadata payload and rows shared by ledger sources and SNS/ICRC reports.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

///
/// IcrcTokenData
///
/// Source-layer token metadata returned by an ICRC ledger.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTokenData {
    /// Native ledger token name.
    pub token_name: String,
    /// Native ledger token symbol.
    pub token_symbol: String,
    /// Number of decimal places in a base-unit amount.
    pub decimals: u8,
    /// Transfer fee as exact base-10 base-unit text.
    pub transfer_fee: String,
    /// Total supply as exact base-10 base-unit text.
    pub total_supply: String,
    /// Optional minting-account owner in principal text.
    pub minting_account_owner: Option<String>,
    /// Optional raw minting subaccount in lowercase hexadecimal.
    pub minting_account_subaccount_hex: Option<String>,
    /// Standards advertised by the ledger.
    pub supported_standards: Vec<IcrcTokenStandardRow>,
    /// Raw ledger metadata preserving native value kinds.
    pub metadata: Vec<IcrcTokenMetadataRow>,
}

///
/// IcrcTokenStandardRow
///
/// Serializable row for one ICRC standard supported by a ledger.
///

#[cfg_attr(
    any(feature = "icrc-host", feature = "sns-host"),
    derive(ic_platform::candid::contract::CandidType)
)]
#[cfg_attr(
    any(feature = "icrc-host", feature = "sns-host"),
    candid_path("ic_platform::candid::contract")
)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IcrcTokenStandardRow {
    /// Standard name advertised by the ledger.
    pub name: String,
    /// Specification URL advertised by the ledger.
    pub url: String,
}

///
/// IcrcTokenMetadataRow
///
/// Serializable row for one raw ICRC ledger metadata entry.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcTokenMetadataRow {
    /// Native metadata key.
    pub key: String,
    /// Native Candid metadata variant.
    pub value_type: IcrcMetadataValueKind,
    /// Lossless decimal, text, or lowercase hexadecimal JSON string.
    pub value: JsonValue,
}

///
/// IcrcMetadataValueKind
///
/// Native ICRC-1 metadata value variant represented by a report row.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IcrcMetadataValueKind {
    /// An arbitrary-precision unsigned integer.
    Nat,
    /// An arbitrary-precision signed integer.
    Int,
    /// A Unicode text value.
    Text,
    /// An opaque byte sequence.
    Blob,
}

impl IcrcMetadataValueKind {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nat => "nat",
            Self::Int => "int",
            Self::Text => "text",
            Self::Blob => "blob",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_value_kind_labels_round_trip() {
        for (kind, label) in [
            (IcrcMetadataValueKind::Nat, "nat"),
            (IcrcMetadataValueKind::Int, "int"),
            (IcrcMetadataValueKind::Text, "text"),
            (IcrcMetadataValueKind::Blob, "blob"),
        ] {
            assert_eq!(
                serde_json::to_string(&kind).unwrap(),
                format!("\"{label}\"")
            );
            assert_eq!(
                serde_json::from_str::<IcrcMetadataValueKind>(&format!("\"{label}\"")).unwrap(),
                kind
            );
            assert_eq!(kind.as_str(), label);
        }
    }
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
    /// Index discovery is unavailable; source families own transport-error policy.
    Unavailable(String),
}
