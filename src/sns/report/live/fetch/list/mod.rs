//! Module: sns::report::live::fetch::list
//!
//! Responsibility: fetch deployed SNS inventory from SNS-W.
//! Does not own: report assembly, command parsing, cache IO, or rendering.
//! Boundary: queries SNS-W and root metadata into source-layer SNS list data.

mod metadata;

use super::block_on_sns;
use crate::sns::report::live::{
    convert::mainnet_sns_canisters_from_deployed_sns,
    query::{principal_from_text, query_canister, sns_agent},
    types::{ListDeployedSnsesRequest, ListDeployedSnsesResponse},
};
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SnsFetchRequest, SnsHostError,
    source::{MainnetSnsCanisters, MainnetSnsList},
};
use crate::subnet_catalog::MAINNET_NETWORK;
use ic_agent::Agent;
use metadata::fetch_mainnet_sns_metadata_rows;

/// Fetch the current deployed SNS list from mainnet SNS-W.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_list(
    request: &SnsFetchRequest,
) -> Result<MainnetSnsList, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_list_async(request))
}

async fn fetch_mainnet_sns_list_async(
    request: &SnsFetchRequest,
) -> Result<MainnetSnsList, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let sns_wasm_canister =
        principal_from_text(MAINNET_SNS_WASM_CANISTER_ID, "sns_wasm_canister_id")?;
    let response: ListDeployedSnsesResponse = query_canister(
        &agent,
        &sns_wasm_canister,
        "list_deployed_snses",
        "ListDeployedSnsesRequest",
        "ListDeployedSnsesResponse",
        &ListDeployedSnsesRequest {},
    )
    .await?;
    mainnet_sns_list_from_response(&agent, request, response).await
}

async fn mainnet_sns_list_from_response(
    agent: &Agent,
    request: &SnsFetchRequest,
    response: ListDeployedSnsesResponse,
) -> Result<MainnetSnsList, SnsHostError> {
    let sns_canisters = response
        .instances
        .into_iter()
        .map(mainnet_sns_canisters_from_deployed_sns)
        .collect::<Result<Vec<MainnetSnsCanisters>, _>>()?;
    let sns_instances = fetch_mainnet_sns_metadata_rows(agent, sns_canisters).await?;
    Ok(MainnetSnsList {
        network: MAINNET_NETWORK.to_string(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        sns_instances,
    })
}
