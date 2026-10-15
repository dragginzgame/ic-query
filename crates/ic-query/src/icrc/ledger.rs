//! Module: icrc::ledger
//!
//! Responsibility: shared live ICRC ledger wire types and metadata conversion.
//! Does not own: CLI parsing, command-specific reports, SNS lookup, or text rendering.
//! Boundary: keeps reusable ICRC ledger mechanics independent from report DTOs.

use crate::{
    candid_call::{CandidCallError, query_no_args},
    hex::hex_bytes,
    icrc::IcrcMetadataValueKind,
};
use candid::{CandidType, Deserialize, Int, Nat, Principal};
use ic_agent::Agent;
use serde_json::Value as JsonValue;

///
/// IcrcLedgerTokenMetadata
///
/// Raw ICRC-1 token metadata fetched from one ledger canister.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcLedgerTokenMetadata {
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub transfer_fee: String,
    pub total_supply: String,
    pub minting_account_owner: Option<String>,
    pub minting_account_subaccount_hex: Option<String>,
    pub supported_standards: Vec<IcrcLedgerStandardRow>,
    pub metadata: Vec<IcrcLedgerMetadataRow>,
}

///
/// IcrcLedgerStandardRow
///
/// Generic row for one ICRC standard supported by a ledger.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcLedgerStandardRow {
    pub name: String,
    pub url: String,
}

///
/// IcrcLedgerMetadataRow
///
/// Generic row for one ICRC ledger metadata key/value pair.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcLedgerMetadataRow {
    pub key: String,
    pub value_type: IcrcMetadataValueKind,
    pub value: JsonValue,
}

///
/// IcrcAccount
///
/// Candid ICRC account argument used by ledger balance calls.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct IcrcAccount {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

///
/// IcrcMetadataValue
///
/// Candid ICRC metadata value returned by ledgers.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum IcrcMetadataValue {
    Nat(Nat),
    Int(Int),
    Text(String),
    Blob(Vec<u8>),
}

///
/// IcrcAllowanceArgs
///
/// Candid ICRC-2 allowance request argument.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[cfg(feature = "icrc-host")]
pub struct IcrcAllowanceArgs {
    pub account: IcrcAccount,
    pub spender: IcrcAccount,
}

///
/// IcrcAllowance
///
/// Candid ICRC-2 allowance response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[cfg(feature = "icrc-host")]
pub struct IcrcAllowance {
    pub allowance: Nat,
    pub expires_at: Option<u64>,
}

///
/// GetIndexPrincipalResult
///
/// Candid result returned by ICRC-106 index discovery.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum GetIndexPrincipalResult {
    Ok(Principal),
    Err(GetIndexPrincipalError),
}

///
/// GetIndexPrincipalError
///
/// Candid error returned by ICRC-106 index discovery.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum GetIndexPrincipalError {
    IndexPrincipalNotSet,
    GenericError {
        error_code: Nat,
        description: String,
    },
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcSupportedStandard {
    name: String,
    url: String,
}

#[cfg(feature = "icrc-host")]
pub fn principal_from_text(
    value: &str,
    field: &'static str,
) -> Result<Principal, crate::icrc::IcrcError> {
    Principal::from_text(value).map_err(|err| crate::icrc::IcrcError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}

pub async fn fetch_icrc1_token_metadata<E>(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<IcrcLedgerTokenMetadata, E>
where
    E: CandidCallError,
{
    let (
        token_name,
        token_symbol,
        decimals,
        transfer_fee,
        total_supply,
        minting_account,
        supported_standards,
        metadata,
    ) = futures::try_join!(
        query_no_args::<String, E>(agent, ledger_canister, "icrc1_name"),
        query_no_args::<String, E>(agent, ledger_canister, "icrc1_symbol"),
        query_no_args::<u8, E>(agent, ledger_canister, "icrc1_decimals"),
        query_no_args::<Nat, E>(agent, ledger_canister, "icrc1_fee"),
        query_no_args::<Nat, E>(agent, ledger_canister, "icrc1_total_supply"),
        query_no_args::<Option<IcrcAccount>, E>(agent, ledger_canister, "icrc1_minting_account"),
        fetch_icrc_supported_standards::<E>(agent, ledger_canister),
        query_no_args::<Vec<(String, IcrcMetadataValue)>, E>(
            agent,
            ledger_canister,
            "icrc1_metadata"
        ),
    )?;

    Ok(IcrcLedgerTokenMetadata {
        token_name,
        token_symbol,
        decimals,
        transfer_fee: nat_text(&transfer_fee),
        total_supply: nat_text(&total_supply),
        minting_account_owner: minting_account
            .as_ref()
            .map(|account| account.owner.to_text()),
        minting_account_subaccount_hex: minting_account
            .as_ref()
            .and_then(|account| account.subaccount.as_deref())
            .map(hex_bytes),
        supported_standards,
        metadata: metadata
            .into_iter()
            .map(|(key, value)| metadata_row(key, value))
            .collect(),
    })
}

pub async fn fetch_icrc_supported_standards<E>(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<Vec<IcrcLedgerStandardRow>, E>
where
    E: CandidCallError,
{
    let supported_standards: Vec<IcrcSupportedStandard> =
        query_no_args(agent, ledger_canister, "icrc1_supported_standards").await?;
    Ok(supported_standards
        .into_iter()
        .map(|standard| IcrcLedgerStandardRow {
            name: standard.name,
            url: standard.url,
        })
        .collect())
}

/// Return plain decimal text for raw Candid naturals without display separators.
pub fn nat_text(value: &Nat) -> String {
    value.0.to_str_radix(10)
}

pub fn metadata_row(key: String, value: IcrcMetadataValue) -> IcrcLedgerMetadataRow {
    let (value_type, value) = match value {
        IcrcMetadataValue::Nat(value) => (IcrcMetadataValueKind::Nat, nat_text(&value)),
        IcrcMetadataValue::Int(value) => (IcrcMetadataValueKind::Int, value.0.to_str_radix(10)),
        IcrcMetadataValue::Text(value) => (IcrcMetadataValueKind::Text, value),
        IcrcMetadataValue::Blob(value) => (IcrcMetadataValueKind::Blob, hex_bytes(&value)),
    };
    IcrcLedgerMetadataRow {
        key,
        value_type,
        value: JsonValue::String(value),
    }
}

#[must_use]
pub fn index_principal_error_text(error: GetIndexPrincipalError) -> String {
    match error {
        GetIndexPrincipalError::IndexPrincipalNotSet => "index principal not set".to_string(),
        GetIndexPrincipalError::GenericError {
            error_code,
            description,
        } => format!("generic error {error_code}: {description}"),
    }
}

#[cfg(all(test, feature = "icrc-host"))]
mod tests {
    use super::*;

    #[test]
    fn metadata_conversion_preserves_native_kind_and_value() {
        for (raw, kind, value) in [
            (
                IcrcMetadataValue::Nat(Nat::from(7_u8)),
                IcrcMetadataValueKind::Nat,
                "7",
            ),
            (
                IcrcMetadataValue::Int(Int::from(-2_i8)),
                IcrcMetadataValueKind::Int,
                "-2",
            ),
            (
                IcrcMetadataValue::Nat("18446744073709551616".parse().unwrap()),
                IcrcMetadataValueKind::Nat,
                "18446744073709551616",
            ),
            (
                IcrcMetadataValue::Int("-18446744073709551616".parse().unwrap()),
                IcrcMetadataValueKind::Int,
                "-18446744073709551616",
            ),
            (
                IcrcMetadataValue::Text("token".to_string()),
                IcrcMetadataValueKind::Text,
                "token",
            ),
            (
                IcrcMetadataValue::Blob(vec![0x01, 0xab]),
                IcrcMetadataValueKind::Blob,
                "01ab",
            ),
        ] {
            let row = metadata_row("key".to_string(), raw);

            assert_eq!(row.key, "key");
            assert_eq!(row.value_type, kind);
            assert_eq!(row.value, JsonValue::String(value.to_string()));
        }
    }
}
