//! Module: icrc::live
//!
//! Responsibility: build ICRC reports and query live generic ICRC ledgers.
//! Does not own: command parsing, text rendering, or cache behavior.
//! Boundary: keeps live Candid calls behind a source trait for fixture tests.

use crate::{
    icrc::{
        ledger::{
            IcrcAccount, IcrcAllowance, IcrcAllowanceArgs, IcrcLedgerError, IcrcLedgerMetadataRow,
            IcrcLedgerStandardRow, IcrcLedgerTokenMetadata, fetch_icrc1_token_metadata, ic_agent,
            principal_from_text, query_ledger, query_ledger_arg,
        },
        model::{
            IcrcAllowanceData, IcrcAllowanceReport, IcrcAllowanceRequest, IcrcBalanceData,
            IcrcBalanceReport, IcrcBalanceRequest, IcrcError, IcrcTokenData, IcrcTokenMetadataRow,
            IcrcTokenReport, IcrcTokenRequest, IcrcTokenStandardRow, normalize_subaccount_hex,
            subaccount_bytes_from_hex,
        },
    },
    runtime::block_on_current_thread,
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::{Nat, Principal};
use ic_agent::Agent;

pub(in crate::icrc) const ICRC_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_BALANCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub(in crate::icrc) const ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION: u32 = 1;
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
/// Source contract for fetching generic ICRC ledger metadata, balances, and allowances.
///
pub(in crate::icrc) trait IcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError>;

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError>;

    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError>;
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

    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        block_on_current_thread(fetch_allowance_async(request))?
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

pub(in crate::icrc) fn build_icrc_allowance_report(
    request: &IcrcAllowanceRequest,
) -> Result<IcrcAllowanceReport, IcrcError> {
    build_icrc_allowance_report_with_source(request, &LiveIcrcSource)
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

pub(in crate::icrc) fn build_icrc_allowance_report_with_source(
    request: &IcrcAllowanceRequest,
    source: &dyn IcrcSource,
) -> Result<IcrcAllowanceReport, IcrcError> {
    let request = IcrcAllowanceRequest {
        account_subaccount_hex: request
            .account_subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
        spender_subaccount_hex: request
            .spender_subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
        ..request.clone()
    };
    let allowance = source.fetch_allowance(&request)?;
    Ok(IcrcAllowanceReport {
        schema_version: ICRC_ALLOWANCE_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        account_owner: request.account_owner,
        account_subaccount_hex: request.account_subaccount_hex,
        spender_owner: request.spender_owner,
        spender_subaccount_hex: request.spender_subaccount_hex,
        fetched_at: format_utc_timestamp_secs(request.now_unix_secs),
        source_endpoint: request.source_endpoint,
        fetched_by: ICRC_FETCHED_BY.to_string(),
        token_symbol: allowance.token_symbol,
        decimals: allowance.decimals,
        allowance: allowance.allowance,
        expires_at_unix_nanos: allowance.expires_at_unix_nanos,
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
    let account = account_from_parts(
        &request.account_owner,
        request.subaccount_hex.as_deref(),
        "account_owner",
    )?;
    let (token_symbol, decimals) = query_token_display_fields(&agent, &ledger_canister).await?;
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

async fn fetch_allowance_async(
    request: &IcrcAllowanceRequest,
) -> Result<IcrcAllowanceData, IcrcError> {
    let agent = ic_agent::<IcrcError>(&request.source_endpoint)?;
    let ledger_canister =
        principal_from_text::<IcrcError>(&request.ledger_canister_id, "ledger_canister_id")?;
    let allowance_args = IcrcAllowanceArgs {
        account: account_from_parts(
            &request.account_owner,
            request.account_subaccount_hex.as_deref(),
            "account_owner",
        )?,
        spender: account_from_parts(
            &request.spender_owner,
            request.spender_subaccount_hex.as_deref(),
            "spender_owner",
        )?,
    };
    let (token_symbol, decimals) = query_token_display_fields(&agent, &ledger_canister).await?;
    let allowance = query_ledger_arg::<IcrcAllowanceArgs, IcrcAllowance, IcrcError>(
        &agent,
        &ledger_canister,
        "icrc2_allowance",
        &allowance_args,
    )
    .await?;

    Ok(IcrcAllowanceData {
        token_symbol,
        decimals,
        allowance: allowance.allowance.to_string(),
        expires_at_unix_nanos: allowance
            .expires_at
            .map(|expires_at| expires_at.to_string()),
    })
}

async fn query_token_display_fields(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<(String, u8), IcrcError> {
    let token_symbol =
        query_ledger::<String, IcrcError>(agent, ledger_canister, "icrc1_symbol").await?;
    let decimals = query_ledger::<u8, IcrcError>(agent, ledger_canister, "icrc1_decimals").await?;
    Ok((token_symbol, decimals))
}

fn account_from_parts(
    owner: &str,
    subaccount_hex: Option<&str>,
    owner_field: &'static str,
) -> Result<IcrcAccount, IcrcError> {
    Ok(IcrcAccount {
        owner: principal_from_text::<IcrcError>(owner, owner_field)?,
        subaccount: subaccount_hex.map(subaccount_bytes_from_hex).transpose()?,
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
