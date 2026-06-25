//! Module: sns::report::live::fetch::token
//!
//! Responsibility: fetch SNS ledger token metadata.
//! Does not own: lookup resolution, report assembly, amount formatting, or rendering.
//! Boundary: queries one resolved SNS ledger for ICRC metadata and index details.

use super::block_on_sns;
use crate::icrc::ledger::{
    GetIndexPrincipalResult, IcrcLedgerMetadataRow, IcrcLedgerStandardRow, IcrcLedgerTokenMetadata,
    fetch_icrc1_token_metadata, index_principal_error_text, query_ledger,
};
use crate::sns::report::live::query::{principal_from_text, sns_agent};
use crate::sns::report::{
    SnsHostError, SnsTokenMetadataRow, SnsTokenStandardRow,
    source::{MainnetSns, MainnetSnsToken, SnsFetchRequest},
};

/// Fetch token metadata for one resolved mainnet SNS ledger.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_token(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_token_async(request, sns))
}

async fn fetch_mainnet_sns_token_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let ledger_canister = principal_from_text(&sns.ledger_canister_id, "ledger_canister_id")?;
    let token = fetch_icrc1_token_metadata::<SnsHostError>(&agent, &ledger_canister).await?;
    let (ledger_index_canister_id, ledger_index_error) = match query_ledger::<
        GetIndexPrincipalResult,
        SnsHostError,
    >(
        &agent,
        &ledger_canister,
        "icrc106_get_index_principal",
    )
    .await
    {
        Ok(GetIndexPrincipalResult::Ok(principal)) => (Some(principal.to_text()), None),
        Ok(GetIndexPrincipalResult::Err(error)) => (None, Some(index_principal_error_text(error))),
        Err(error) => (None, Some(error.to_string())),
    };

    Ok(mainnet_sns_token_from_ledger(
        token,
        ledger_index_canister_id,
        ledger_index_error,
    ))
}

fn mainnet_sns_token_from_ledger(
    token: IcrcLedgerTokenMetadata,
    ledger_index_canister_id: Option<String>,
    ledger_index_error: Option<String>,
) -> MainnetSnsToken {
    MainnetSnsToken {
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        ledger_index_canister_id,
        ledger_index_error,
        supported_standards: token
            .supported_standards
            .into_iter()
            .map(sns_standard_row_from_ledger)
            .collect(),
        metadata: token
            .metadata
            .into_iter()
            .map(sns_metadata_row_from_ledger)
            .collect(),
    }
}

fn sns_standard_row_from_ledger(row: IcrcLedgerStandardRow) -> SnsTokenStandardRow {
    SnsTokenStandardRow {
        name: row.name,
        url: row.url,
    }
}

fn sns_metadata_row_from_ledger(row: IcrcLedgerMetadataRow) -> SnsTokenMetadataRow {
    SnsTokenMetadataRow {
        key: row.key,
        value_type: row.value_type,
        value: row.value,
    }
}
