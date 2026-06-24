//! Module: icrc::live
//!
//! Responsibility: build ICRC reports and query live generic ICRC ledgers.
//! Does not own: command parsing, text rendering, or cache behavior.
//! Boundary: keeps live Candid calls behind a source trait for fixture tests.

use crate::{
    icrc::{
        ledger::{
            IcrcAccount, IcrcLedgerError, IcrcLedgerMetadataRow, IcrcLedgerStandardRow,
            IcrcLedgerTokenMetadata, fetch_icrc1_token_metadata, ic_agent, principal_from_text,
            query_ledger, query_ledger_arg,
        },
        model::{
            IcrcBalanceData, IcrcBalanceReport, IcrcBalanceRequest, IcrcError, IcrcTokenData,
            IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenRequest, IcrcTokenStandardRow,
            normalize_subaccount_hex, subaccount_bytes_from_hex,
        },
    },
    runtime::block_on_current_thread,
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::Nat;

pub(in crate::icrc) const ICRC_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BALANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_FETCHED_BY: &str = "ic-query";

impl IcrcLedgerError for IcrcError {
    fn agent_build(endpoint: &str, reason: String) -> Self {
        Self::AgentBuild {
            endpoint: endpoint.to_string(),
            reason,
        }
    }

    fn invalid_principal(field: &'static str, reason: String) -> Self {
        Self::InvalidPrincipal { field, reason }
    }

    fn candid_encode(message: &'static str, reason: String) -> Self {
        Self::CandidEncode { message, reason }
    }

    fn agent_call(method: &'static str, reason: String) -> Self {
        Self::AgentCall { method, reason }
    }

    fn candid_decode(message: &'static str, reason: String) -> Self {
        Self::CandidDecode { message, reason }
    }
}

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
    let agent = ic_agent::<IcrcError>(&request.source_endpoint)?;
    let ledger_canister =
        principal_from_text::<IcrcError>(&request.ledger_canister_id, "ledger_canister_id")?;
    fetch_icrc1_token_metadata::<IcrcError>(&agent, &ledger_canister)
        .await
        .map(token_data_from_ledger)
}

async fn fetch_balance_async(request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
    let agent = ic_agent::<IcrcError>(&request.source_endpoint)?;
    let ledger_canister =
        principal_from_text::<IcrcError>(&request.ledger_canister_id, "ledger_canister_id")?;
    let account = IcrcAccount {
        owner: principal_from_text::<IcrcError>(&request.account_owner, "account_owner")?,
        subaccount: request
            .subaccount_hex
            .as_deref()
            .map(subaccount_bytes_from_hex)
            .transpose()?,
    };
    let token_symbol =
        query_ledger::<String, IcrcError>(&agent, &ledger_canister, "icrc1_symbol").await?;
    let decimals =
        query_ledger::<u8, IcrcError>(&agent, &ledger_canister, "icrc1_decimals").await?;
    let balance: Nat = query_ledger_arg::<IcrcAccount, Nat, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc1_balance_of",
        &account,
    )
    .await?;

    Ok(IcrcBalanceData {
        token_symbol,
        decimals,
        balance: balance.to_string(),
    })
}

fn token_data_from_ledger(token: IcrcLedgerTokenMetadata) -> IcrcTokenData {
    IcrcTokenData {
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        supported_standards: token
            .supported_standards
            .into_iter()
            .map(token_standard_row_from_ledger)
            .collect(),
        metadata: token
            .metadata
            .into_iter()
            .map(token_metadata_row_from_ledger)
            .collect(),
    }
}

fn token_standard_row_from_ledger(row: IcrcLedgerStandardRow) -> IcrcTokenStandardRow {
    IcrcTokenStandardRow {
        name: row.name,
        url: row.url,
    }
}

fn token_metadata_row_from_ledger(row: IcrcLedgerMetadataRow) -> IcrcTokenMetadataRow {
    IcrcTokenMetadataRow {
        key: row.key,
        value_type: row.value_type,
        value: row.value,
    }
}
