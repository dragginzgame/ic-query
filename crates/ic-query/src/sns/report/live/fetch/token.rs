//! Module: sns::report::live::fetch::token
//!
//! Responsibility: fetch SNS ledger token metadata.
//! Does not own: lookup resolution, report assembly, amount formatting, or rendering.
//! Boundary: queries one resolved SNS ledger for ICRC metadata and index details.
use crate::icrc::ledger::{
    fetch_icrc1_token_metadata, index_principal_error_text, query_index_principal,
};
use crate::sns::report::live::query::{principal_from_text, sns_agent};
use crate::sns::report::{
    SnsHostError,
    source::{MainnetSns, MainnetSnsToken, SnsSourceRequest},
};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_token(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let ledger_canister = principal_from_text(&sns.ledger_canister_id, "ledger_canister_id")?;
    let token = Box::pin(fetch_icrc1_token_metadata::<SnsHostError>(
        agent,
        &ledger_canister,
    ))
    .await?;
    let (ledger_index_canister_id, ledger_index_error) =
        match query_index_principal::<SnsHostError>(agent, &ledger_canister).await {
            Ok(Ok(principal)) => (Some(principal.to_text()), None),
            Ok(Err(error)) => (None, Some(index_principal_error_text(error))),
            Err(error) => (None, Some(error.to_string())),
        };

    Ok(MainnetSnsToken {
        token,
        ledger_index_canister_id,
        ledger_index_error,
    })
}
