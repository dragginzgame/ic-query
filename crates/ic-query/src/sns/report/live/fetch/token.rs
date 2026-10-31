//! Module: sns::report::live::fetch::token
//!
//! Responsibility: fetch SNS ledger token metadata.
//! Does not own: lookup resolution, report assembly, amount formatting, or rendering.
//! Boundary: queries one resolved SNS ledger for ICRC metadata and index details.
use crate::icrc::IcrcIndexData;
use crate::icrc::ledger::{
    fetch_icrc1_token_metadata, index_principal_error_text, query_index_principal,
};
use crate::report::CollectionSourceRequest;
use crate::sns::report::live::query::{principal_from_text, sns_agent};
use crate::sns::report::{
    SnsHostError,
    source::{MainnetSns, MainnetSnsToken},
};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_token(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let ledger_canister =
        principal_from_text(&sns.canisters.ledger_canister_id, "ledger_canister_id")?;
    let token = Box::pin(fetch_icrc1_token_metadata::<SnsHostError>(
        agent,
        &ledger_canister,
    ))
    .await?;
    let ledger_index = match query_index_principal::<SnsHostError>(agent, &ledger_canister).await {
        Ok(Ok(principal)) => IcrcIndexData::Found(principal.to_text()),
        Ok(Err(error)) => IcrcIndexData::Unavailable(index_principal_error_text(error)),
        Err(error) => IcrcIndexData::Unavailable(error.to_string()),
    };

    Ok(MainnetSnsToken {
        token,
        ledger_index,
    })
}
