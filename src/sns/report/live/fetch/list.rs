use super::block_on_sns;
use crate::sns::report::live::{
    convert::{
        mainnet_sns_canisters_from_deployed_sns, mainnet_sns_from_canisters_and_metadata,
        metadata_error_summary,
    },
    query::{principal_from_text, query_canister, sns_agent},
    types::{
        GetMetadataRequest, GetMetadataResponse, ListDeployedSnsesRequest,
        ListDeployedSnsesResponse,
    },
};
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SNS_METADATA_CONCURRENCY, SnsHostError,
    source::{MainnetSns, MainnetSnsCanisters, MainnetSnsList, SnsFetchRequest},
};
use crate::subnet_catalog::MAINNET_NETWORK;
use candid::Principal;
use futures::{StreamExt, stream};
use ic_agent::Agent;

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
        .collect::<Result<Vec<_>, _>>()?;
    let fetched = stream::iter(
        sns_canisters
            .into_iter()
            .map(|sns| fetch_mainnet_sns_metadata(agent, sns)),
    )
    .buffered(SNS_METADATA_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;
    let mut sns_instances = Vec::with_capacity(fetched.len());
    for sns in fetched {
        sns_instances.push(sns?);
    }
    Ok(MainnetSnsList {
        network: MAINNET_NETWORK.to_string(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        sns_instances,
    })
}

async fn fetch_mainnet_sns_metadata(
    agent: &Agent,
    sns: MainnetSnsCanisters,
) -> Result<MainnetSns, SnsHostError> {
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let (metadata, metadata_error) =
        match fetch_governance_metadata(agent, &governance_canister).await {
            Ok(metadata) => (metadata, None),
            Err(err) => match metadata_error_summary(&err) {
                Some(summary) => (GetMetadataResponse::default(), Some(summary)),
                None => return Err(err),
            },
        };
    Ok(mainnet_sns_from_canisters_and_metadata(
        sns,
        metadata,
        metadata_error,
    ))
}

async fn fetch_governance_metadata(
    agent: &Agent,
    governance_canister: &Principal,
) -> Result<GetMetadataResponse, SnsHostError> {
    query_canister(
        agent,
        governance_canister,
        "get_metadata",
        "GetMetadataRequest",
        "GetMetadataResponse",
        &GetMetadataRequest {},
    )
    .await
}
