//! Module: icrc::ledger
//!
//! Responsibility: shared live ICRC ledger wire types and metadata conversion.
//! Does not own: CLI parsing, command-specific reports, SNS lookup, or text rendering.
//! Boundary: keeps reusable ICRC ledger mechanics independent from report DTOs.

use crate::{
    candid_call::{CandidCallError, query_no_args},
    hex::hex_bytes,
    icrc::{IcrcMetadataValueKind, IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenStandardRow},
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::{CandidType, Nat, Principal, serde::Deserialize};
use ic_platform::icrc_ledger_types::{Account as LedgerAccount, MetadataValue};
use serde_json::Value as JsonValue;

pub const ICRC106_GET_INDEX_PRINCIPAL_METHOD: &str = "icrc106_get_index_principal";

///
/// GetIndexPrincipalError
///
/// Candid error returned by ICRC-106 index discovery.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub enum GetIndexPrincipalError {
    IndexPrincipalNotSet,
    GenericError {
        error_code: Nat,
        description: String,
    },
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
) -> Result<IcrcTokenData, E>
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
    ) = futures_util::try_join!(
        query_no_args::<String, E>(agent, ledger_canister, "icrc1_name"),
        query_no_args::<String, E>(agent, ledger_canister, "icrc1_symbol"),
        query_no_args::<u8, E>(agent, ledger_canister, "icrc1_decimals"),
        query_no_args::<Nat, E>(agent, ledger_canister, "icrc1_fee"),
        query_no_args::<Nat, E>(agent, ledger_canister, "icrc1_total_supply"),
        query_no_args::<Option<LedgerAccount>, E>(agent, ledger_canister, "icrc1_minting_account"),
        fetch_icrc_supported_standards::<E>(agent, ledger_canister),
        query_no_args::<Vec<(String, MetadataValue)>, E>(agent, ledger_canister, "icrc1_metadata"),
    )?;

    Ok(IcrcTokenData {
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
            .and_then(|account| account.subaccount.as_ref().map(<[u8; 32]>::as_slice))
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
) -> Result<Vec<IcrcTokenStandardRow>, E>
where
    E: CandidCallError,
{
    query_no_args(agent, ledger_canister, "icrc1_supported_standards").await
}

/// Query native ICRC-106 discovery while callers retain their failure policy.
pub async fn query_index_principal<E: CandidCallError>(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<Result<Principal, GetIndexPrincipalError>, E> {
    query_no_args::<Result<Principal, GetIndexPrincipalError>, E>(
        agent,
        ledger_canister,
        ICRC106_GET_INDEX_PRINCIPAL_METHOD,
    )
    .await
}

/// Return plain decimal text for raw Candid naturals without display separators.
pub fn nat_text(value: &Nat) -> String {
    value.0.to_str_radix(10)
}

pub fn metadata_row(key: String, value: MetadataValue) -> IcrcTokenMetadataRow {
    let (value_type, value) = match value {
        MetadataValue::Nat(value) => (IcrcMetadataValueKind::Nat, nat_text(&value)),
        MetadataValue::Int(value) => (IcrcMetadataValueKind::Int, value.0.to_str_radix(10)),
        MetadataValue::Text(value) => (IcrcMetadataValueKind::Text, value),
        MetadataValue::Blob(value) => (IcrcMetadataValueKind::Blob, hex_bytes(&value)),
    };
    IcrcTokenMetadataRow {
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
    use ic_platform::candid::contract::Int;

    #[test]
    fn native_index_and_standard_replies_decode_without_projection() {
        // Independent fixtures: didc encode of native Ok/Err variants and standard records.
        for (encoded, expected) in [
            (
                "4449444c016b01bc8a01680100000100",
                Ok(Principal::management_canister()),
            ),
            (
                "4449444c026b01c5fed201016b01cbfde8ed047f01000000",
                Err(GetIndexPrincipalError::IndexPrincipalNotSet),
            ),
            (
                "4449444c036b01c5fed201016b01d1c4987c026c02fc91f4f80571c498b1b50d7d0100000007666978747572652a",
                Err(GetIndexPrincipalError::GenericError {
                    error_code: Nat::from(42_u8),
                    description: "fixture".to_string(),
                }),
            ),
        ] {
            let bytes = crate::hex::decode_hex(encoded).unwrap();
            let actual: Result<Principal, GetIndexPrincipalError> =
                crate::candid_decode::decode_reply(&bytes).unwrap();
            assert_eq!(actual, expected);
        }
        let bytes = crate::hex::decode_hex("4449444c026d016c02efd6e40271cbe4fdc704710100011c68747470733a2f2f6578616d706c652e636f6d2f7374616e6461726406494352432d31").unwrap();
        let standards: Vec<IcrcTokenStandardRow> =
            crate::candid_decode::decode_reply(&bytes).unwrap();
        assert_eq!(
            standards,
            vec![IcrcTokenStandardRow {
                name: "ICRC-1".to_string(),
                url: "https://example.com/standard".to_string()
            }]
        );
    }

    #[test]
    fn metadata_conversion_preserves_native_kind_and_value() {
        for (raw, kind, value) in [
            (
                MetadataValue::Nat(Nat::from(7_u8)),
                IcrcMetadataValueKind::Nat,
                "7",
            ),
            (
                MetadataValue::Int(Int::from(-2_i8)),
                IcrcMetadataValueKind::Int,
                "-2",
            ),
            (
                MetadataValue::Nat("18446744073709551616".parse().unwrap()),
                IcrcMetadataValueKind::Nat,
                "18446744073709551616",
            ),
            (
                MetadataValue::Int("-18446744073709551616".parse().unwrap()),
                IcrcMetadataValueKind::Int,
                "-18446744073709551616",
            ),
            (
                MetadataValue::Text("token".to_string()),
                IcrcMetadataValueKind::Text,
                "token",
            ),
            (
                MetadataValue::Blob(vec![0x01, 0xab].into()),
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
