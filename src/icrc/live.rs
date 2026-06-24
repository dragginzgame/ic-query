//! Module: icrc::live
//!
//! Responsibility: build ICRC reports and query live generic ICRC ledgers.
//! Does not own: command parsing, text rendering, or cache behavior.
//! Boundary: keeps live Candid calls behind a source trait for fixture tests.

use crate::{
    hex::hex_bytes,
    icrc::model::{
        IcrcBalanceData, IcrcBalanceReport, IcrcBalanceRequest, IcrcError, IcrcTokenData,
        IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenRequest, IcrcTokenStandardRow,
        normalize_subaccount_hex, subaccount_bytes_from_hex,
    },
    runtime::block_on_current_thread,
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::{CandidType, Decode, Deserialize, Encode, Int, Nat, Principal};
use ic_agent::Agent;
use serde_json::Value as JsonValue;

pub(in crate::icrc) const ICRC_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BALANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_FETCHED_BY: &str = "ic-query";
const ICRC_LOGO_METADATA_KEY: &str = "icrc1:logo";

///
/// IcrcSource
///
/// Source contract for fetching generic ICRC ledger metadata and balances.
///
pub(in crate::icrc) trait IcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError>;

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError>;
}

///
/// LiveIcrcSource
///
/// Source implementation backed by live ICRC ledger canister queries.
///
pub(in crate::icrc) struct LiveIcrcSource;

impl IcrcSource for LiveIcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
        block_on_current_thread(fetch_token_async(request))?
    }

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        block_on_current_thread(fetch_balance_async(request))?
    }
}

pub(in crate::icrc) fn build_icrc_token_report(
    request: &IcrcTokenRequest,
) -> Result<IcrcTokenReport, IcrcError> {
    build_icrc_token_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_balance_report(
    request: &IcrcBalanceRequest,
) -> Result<IcrcBalanceReport, IcrcError> {
    build_icrc_balance_report_with_source(request, &LiveIcrcSource)
}

pub(in crate::icrc) fn build_icrc_token_report_with_source(
    request: &IcrcTokenRequest,
    source: &dyn IcrcSource,
) -> Result<IcrcTokenReport, IcrcError> {
    let token = source.fetch_token(request)?;
    Ok(IcrcTokenReport {
        schema_version: ICRC_TOKEN_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id.clone(),
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: ICRC_FETCHED_BY.to_string(),
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        supported_standards: token.supported_standards,
        metadata: token.metadata,
    })
}

pub(in crate::icrc) fn build_icrc_balance_report_with_source(
    request: &IcrcBalanceRequest,
    source: &dyn IcrcSource,
) -> Result<IcrcBalanceReport, IcrcError> {
    let request = IcrcBalanceRequest {
        subaccount_hex: request
            .subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
        ..request.clone()
    };
    let balance = source.fetch_balance(&request)?;
    Ok(IcrcBalanceReport {
        schema_version: ICRC_BALANCE_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint,
        fetched_by: ICRC_FETCHED_BY.to_string(),
        token_symbol: balance.token_symbol,
        decimals: balance.decimals,
        balance: balance.balance,
    })
}

async fn fetch_token_async(request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
    let agent = ic_agent(&request.source_endpoint)?;
    let ledger_canister = principal_from_text(&request.ledger_canister_id, "ledger_canister_id")?;
    let token_name = query_ledger(&agent, &ledger_canister, "icrc1_name").await?;
    let token_symbol = query_ledger(&agent, &ledger_canister, "icrc1_symbol").await?;
    let decimals = query_ledger(&agent, &ledger_canister, "icrc1_decimals").await?;
    let transfer_fee: Nat = query_ledger(&agent, &ledger_canister, "icrc1_fee").await?;
    let total_supply: Nat = query_ledger(&agent, &ledger_canister, "icrc1_total_supply").await?;
    let minting_account: Option<IcrcAccount> =
        query_ledger(&agent, &ledger_canister, "icrc1_minting_account").await?;
    let supported_standards: Vec<IcrcSupportedStandard> =
        query_ledger(&agent, &ledger_canister, "icrc1_supported_standards").await?;
    let metadata: Vec<(String, IcrcMetadataValue)> =
        query_ledger(&agent, &ledger_canister, "icrc1_metadata").await?;

    Ok(IcrcTokenData {
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
            .map(|standard| IcrcTokenStandardRow {
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

async fn fetch_balance_async(request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
    let agent = ic_agent(&request.source_endpoint)?;
    let ledger_canister = principal_from_text(&request.ledger_canister_id, "ledger_canister_id")?;
    let account = IcrcAccount {
        owner: principal_from_text(&request.account_owner, "account_owner")?,
        subaccount: request
            .subaccount_hex
            .as_deref()
            .map(subaccount_bytes_from_hex)
            .transpose()?,
    };
    let token_symbol = query_ledger(&agent, &ledger_canister, "icrc1_symbol").await?;
    let decimals = query_ledger(&agent, &ledger_canister, "icrc1_decimals").await?;
    let balance: Nat =
        query_ledger_arg(&agent, &ledger_canister, "icrc1_balance_of", &account).await?;

    Ok(IcrcBalanceData {
        token_symbol,
        decimals,
        balance: balance.to_string(),
    })
}

async fn query_ledger<T>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
) -> Result<T, IcrcError>
where
    T: for<'de> Deserialize<'de> + CandidType,
{
    let arg = Encode!().map_err(|err| IcrcError::CandidEncode {
        message: method,
        reason: err.to_string(),
    })?;
    query_encoded(agent, ledger_canister, method, arg).await
}

async fn query_ledger_arg<Arg, Response>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
    arg: &Arg,
) -> Result<Response, IcrcError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| IcrcError::CandidEncode {
        message: method,
        reason: err.to_string(),
    })?;
    query_encoded(agent, ledger_canister, method, arg).await
}

async fn query_encoded<T>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
    arg: Vec<u8>,
) -> Result<T, IcrcError>
where
    T: for<'de> Deserialize<'de> + CandidType,
{
    let bytes = agent
        .query(ledger_canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| IcrcError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    Decode!(&bytes, T).map_err(|err| IcrcError::CandidDecode {
        message: method,
        reason: err.to_string(),
    })
}

fn ic_agent(endpoint: &str) -> Result<Agent, IcrcError> {
    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|err| IcrcError::AgentBuild {
            endpoint: endpoint.to_string(),
            reason: err.to_string(),
        })
}

fn principal_from_text(value: &str, field: &'static str) -> Result<Principal, IcrcError> {
    Principal::from_text(value).map_err(|err| IcrcError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}

fn metadata_row(key: String, value: IcrcMetadataValue) -> IcrcTokenMetadataRow {
    if key == ICRC_LOGO_METADATA_KEY {
        return IcrcTokenMetadataRow {
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
    IcrcTokenMetadataRow {
        key,
        value_type: value_type.to_string(),
        value: JsonValue::String(value),
    }
}

fn metadata_value_is_present(value: &IcrcMetadataValue) -> bool {
    match value {
        IcrcMetadataValue::Text(value) => !value.trim().is_empty(),
        IcrcMetadataValue::Blob(value) => !value.is_empty(),
        IcrcMetadataValue::Nat(_) | IcrcMetadataValue::Int(_) => true,
    }
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcAccount {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum IcrcMetadataValue {
    Nat(Nat),
    Int(Int),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcSupportedStandard {
    name: String,
    url: String,
}
