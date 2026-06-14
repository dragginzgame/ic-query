use super::block_on_sns;
use crate::hex::hex_bytes;
use crate::sns::report::live::{
    convert::{index_principal_error_text, metadata_row},
    query::{principal_from_text, query_ledger, sns_agent},
    types::{GetIndexPrincipalResult, IcrcAccount, IcrcMetadataValue, IcrcSupportedStandard},
};
use crate::sns::report::{
    SnsHostError, SnsTokenStandardRow,
    source::{MainnetSns, MainnetSnsToken, SnsFetchRequest},
};
use candid::Nat;

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
    let (ledger_index_canister_id, ledger_index_error) =
        match query_ledger::<GetIndexPrincipalResult>(
            &agent,
            &ledger_canister,
            "icrc106_get_index_principal",
        )
        .await
        {
            Ok(GetIndexPrincipalResult::Ok(principal)) => (Some(principal.to_text()), None),
            Ok(GetIndexPrincipalResult::Err(error)) => {
                (None, Some(index_principal_error_text(error)))
            }
            Err(error) => (None, Some(error.to_string())),
        };

    Ok(MainnetSnsToken {
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
        ledger_index_canister_id,
        ledger_index_error,
        supported_standards: supported_standards
            .into_iter()
            .map(|standard| SnsTokenStandardRow {
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
