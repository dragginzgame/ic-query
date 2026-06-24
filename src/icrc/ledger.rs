//! Module: icrc::ledger
//!
//! Responsibility: shared live ICRC ledger wire types, query helpers, and metadata conversion.
//! Does not own: CLI parsing, command-specific reports, SNS lookup, or text rendering.
//! Boundary: keeps reusable ICRC ledger mechanics independent from report DTOs.

use crate::hex::hex_bytes;
use candid::{CandidType, Deserialize, Encode, Int, Nat, Principal};
use ic_agent::Agent;
use serde_json::Value as JsonValue;

const ICRC_LOGO_METADATA_KEY: &str = "icrc1:logo";

///
/// IcrcLedgerError
///
/// Error adapter implemented by command families that issue ICRC ledger calls.
///

pub trait IcrcLedgerError: Sized {
    fn agent_build(endpoint: &str, reason: String) -> Self;

    fn invalid_principal(field: &'static str, reason: String) -> Self;

    fn candid_encode(message: &'static str, reason: String) -> Self;

    fn agent_call(method: &'static str, reason: String) -> Self;

    fn candid_decode(message: &'static str, reason: String) -> Self;
}

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
    pub value_type: String,
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

pub fn ic_agent<E>(endpoint: &str) -> Result<Agent, E>
where
    E: IcrcLedgerError,
{
    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|err| E::agent_build(endpoint, err.to_string()))
}

pub fn principal_from_text<E>(value: &str, field: &'static str) -> Result<Principal, E>
where
    E: IcrcLedgerError,
{
    Principal::from_text(value).map_err(|err| E::invalid_principal(field, err.to_string()))
}

pub async fn fetch_icrc1_token_metadata<E>(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<IcrcLedgerTokenMetadata, E>
where
    E: IcrcLedgerError,
{
    let token_name = query_ledger(agent, ledger_canister, "icrc1_name").await?;
    let token_symbol = query_ledger(agent, ledger_canister, "icrc1_symbol").await?;
    let decimals = query_ledger(agent, ledger_canister, "icrc1_decimals").await?;
    let transfer_fee: Nat = query_ledger(agent, ledger_canister, "icrc1_fee").await?;
    let total_supply: Nat = query_ledger(agent, ledger_canister, "icrc1_total_supply").await?;
    let minting_account: Option<IcrcAccount> =
        query_ledger(agent, ledger_canister, "icrc1_minting_account").await?;
    let supported_standards: Vec<IcrcSupportedStandard> =
        query_ledger(agent, ledger_canister, "icrc1_supported_standards").await?;
    let metadata: Vec<(String, IcrcMetadataValue)> =
        query_ledger(agent, ledger_canister, "icrc1_metadata").await?;

    Ok(IcrcLedgerTokenMetadata {
        token_name,
        token_symbol,
        decimals,
        transfer_fee: transfer_fee.to_string(),
        total_supply: total_supply.to_string(),
        minting_account_owner: minting_account
            .as_ref()
            .map(|account| account.owner.to_text()),
        minting_account_subaccount_hex: minting_account
            .as_ref()
            .and_then(|account| account.subaccount.as_deref())
            .map(hex_bytes),
        supported_standards: supported_standards
            .into_iter()
            .map(|standard| IcrcLedgerStandardRow {
                name: standard.name,
                url: standard.url,
            })
            .collect(),
        metadata: metadata
            .into_iter()
            .map(|(key, value)| metadata_row(key, value))
            .collect(),
    })
}

pub async fn query_ledger<T, E>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
) -> Result<T, E>
where
    E: IcrcLedgerError,
    T: for<'de> Deserialize<'de> + CandidType,
{
    let arg = Encode!().map_err(|err| E::candid_encode(method, err.to_string()))?;
    query_encoded(agent, ledger_canister, method, arg).await
}

pub async fn query_ledger_arg<Arg, Response, E>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
    arg: &Arg,
) -> Result<Response, E>
where
    Arg: CandidType + Sync,
    E: IcrcLedgerError,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| E::candid_encode(method, err.to_string()))?;
    query_encoded(agent, ledger_canister, method, arg).await
}

pub fn metadata_row(key: String, value: IcrcMetadataValue) -> IcrcLedgerMetadataRow {
    if key == ICRC_LOGO_METADATA_KEY {
        return IcrcLedgerMetadataRow {
            key,
            value_type: "bool".to_string(),
            value: JsonValue::Bool(metadata_value_is_present(&value)),
        };
    }

    let (value_type, value) = match value {
        IcrcMetadataValue::Nat(value) => ("nat", value.to_string()),
        IcrcMetadataValue::Int(value) => ("int", value.to_string()),
        IcrcMetadataValue::Text(value) => ("text", value),
        IcrcMetadataValue::Blob(value) => ("blob", hex_bytes(&value)),
    };
    IcrcLedgerMetadataRow {
        key,
        value_type: value_type.to_string(),
        value: JsonValue::String(value),
    }
}

async fn query_encoded<T, E>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
    arg: Vec<u8>,
) -> Result<T, E>
where
    E: IcrcLedgerError,
    T: for<'de> Deserialize<'de> + CandidType,
{
    let bytes = agent
        .query(ledger_canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| E::agent_call(method, err.to_string()))?;
    candid::decode_one(&bytes).map_err(|err| E::candid_decode(method, err.to_string()))
}

fn metadata_value_is_present(value: &IcrcMetadataValue) -> bool {
    match value {
        IcrcMetadataValue::Text(value) => !value.trim().is_empty(),
        IcrcMetadataValue::Blob(value) => !value.is_empty(),
        IcrcMetadataValue::Nat(_) | IcrcMetadataValue::Int(_) => true,
    }
}
