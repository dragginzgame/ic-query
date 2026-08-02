//! Module: icrc::live::fetch::account
//!
//! Responsibility: query ICRC token and account methods and project their results.
//! Does not own: ledger history, capability probing, source traits, caching, or rendering.
//! Boundary: owns token, balance, allowance, and structured-account live adaptation.

use super::live_query_context;
use crate::icrc::{
    ledger::{
        IcrcAccount, IcrcAllowance, IcrcAllowanceArgs, IcrcLedgerMetadataRow,
        IcrcLedgerStandardRow, IcrcLedgerTokenMetadata, fetch_icrc1_token_metadata,
        principal_from_text, query_ledger, query_ledger_arg,
    },
    model::{
        IcrcAllowanceData, IcrcAllowanceRequest, IcrcBalanceData, IcrcBalanceRequest, IcrcError,
        IcrcLedgerRequest, IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenStandardRow,
        subaccount_bytes_from_hex,
    },
};
use candid::{Nat, Principal};
use ic_agent::Agent;

const ICRC1_SYMBOL_METHOD: &str = "icrc1_symbol";
const ICRC1_DECIMALS_METHOD: &str = "icrc1_decimals";
const ICRC1_BALANCE_OF_METHOD: &str = "icrc1_balance_of";
const ICRC2_ALLOWANCE_METHOD: &str = "icrc2_allowance";

pub(in crate::icrc::live) async fn fetch_token_async(
    request: &IcrcLedgerRequest,
) -> Result<IcrcTokenData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    Box::pin(fetch_icrc1_token_metadata::<IcrcError>(
        &agent,
        &ledger_canister,
    ))
    .await
    .map(token_data_from_ledger)
}

pub(in crate::icrc::live) async fn fetch_balance_async(
    request: &IcrcBalanceRequest,
) -> Result<IcrcBalanceData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
    let account = account_from_parts(
        &request.account_owner,
        request.subaccount_hex.as_deref(),
        "account_owner",
    )?;
    let (token_symbol, decimals) = query_token_display_fields(&agent, &ledger_canister).await?;
    let balance: Nat = query_ledger_arg::<IcrcAccount, Nat, IcrcError>(
        &agent,
        &ledger_canister,
        ICRC1_BALANCE_OF_METHOD,
        &account,
    )
    .await?;

    Ok(IcrcBalanceData {
        token_symbol,
        decimals,
        balance: balance.to_string(),
    })
}

pub(in crate::icrc::live) async fn fetch_allowance_async(
    request: &IcrcAllowanceRequest,
) -> Result<IcrcAllowanceData, IcrcError> {
    let (agent, ledger_canister) =
        live_query_context(&request.source_endpoint, &request.ledger_canister_id)?;
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
        ICRC2_ALLOWANCE_METHOD,
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

pub(in crate::icrc::live) async fn query_token_display_fields(
    agent: &Agent,
    ledger_canister: &Principal,
) -> Result<(String, u8), IcrcError> {
    let (token_symbol, decimals) = futures::try_join!(
        query_ledger::<String, IcrcError>(agent, ledger_canister, ICRC1_SYMBOL_METHOD),
        query_ledger::<u8, IcrcError>(agent, ledger_canister, ICRC1_DECIMALS_METHOD),
    )?;
    Ok((token_symbol, decimals))
}

pub(in crate::icrc::live) fn account_from_parts(
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

pub(super) fn token_standard_row_from_ledger(row: IcrcLedgerStandardRow) -> IcrcTokenStandardRow {
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
