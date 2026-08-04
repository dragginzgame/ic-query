//! Module: sns::report::live::fetch::list
//!
//! Responsibility: fetch deployed SNS inventory and explicitly targeted catalog enrichment.
//! Does not own: report assembly, command parsing, cache IO, or rendering.
//! Boundary: keeps SNS-W inventory separate from bounded Governance metadata and Swap lifecycle queries.

use super::block_on_sns;
use crate::sns::report::live::{
    convert::{
        bounded_query_error_summary, mainnet_sns_canisters_from_deployed_sns,
        mainnet_sns_metadata_from_response,
    },
    query::{principal_from_text, query_canister, sns_agent},
    types::{
        GetLifecycleResponse, GetMetadataRequest, GetMetadataResponse, ListDeployedSnsesRequest,
        ListDeployedSnsesResponse, SnsSwapQueryRequest,
    },
};
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SNS_METADATA_CONCURRENCY, SnsCanisterMethod, SnsHostError,
    SnsSourceRequest, enforce_mainnet_network,
    source::{
        MainnetSnsCanisters, MainnetSnsInventory, MainnetSnsLifecycle, MainnetSnsMetadata,
        sns_swap_lifecycle_name,
    },
};
use crate::subnet_catalog::MAINNET_NETWORK;
use candid::Principal;
use futures::{StreamExt, stream};
use ic_agent::Agent;

/// Fetch the current unenriched deployed-SNS inventory from mainnet SNS-W.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_inventory(
    request: &SnsSourceRequest,
) -> Result<MainnetSnsInventory, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_inventory_async(request))
}

async fn fetch_mainnet_sns_inventory_async(
    request: &SnsSourceRequest,
) -> Result<MainnetSnsInventory, SnsHostError> {
    let agent = sns_agent(request)?;
    let sns_wasm_canister =
        principal_from_text(MAINNET_SNS_WASM_CANISTER_ID, "sns_wasm_canister_id")?;
    let response: ListDeployedSnsesResponse = query_canister(
        &agent,
        &sns_wasm_canister,
        SnsCanisterMethod::ListDeployedSnses.as_str(),
        "ListDeployedSnsesRequest",
        "ListDeployedSnsesResponse",
        &ListDeployedSnsesRequest {},
    )
    .await?;
    mainnet_sns_inventory_from_response(request, response)
}

fn mainnet_sns_inventory_from_response(
    request: &SnsSourceRequest,
    response: ListDeployedSnsesResponse,
) -> Result<MainnetSnsInventory, SnsHostError> {
    let sns_instances = response
        .instances
        .into_iter()
        .map(mainnet_sns_canisters_from_deployed_sns)
        .collect::<Result<Vec<MainnetSnsCanisters>, _>>()?;
    Ok(MainnetSnsInventory {
        network: MAINNET_NETWORK.to_string(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        sns_instances,
    })
}

/// Fetch Governance metadata for exactly the requested deployed-SNS targets.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_metadata(
    request: &SnsSourceRequest,
    targets: &[MainnetSnsCanisters],
) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    block_on_sns(fetch_mainnet_sns_metadata_async(request, targets))
}

async fn fetch_mainnet_sns_metadata_async(
    request: &SnsSourceRequest,
    targets: &[MainnetSnsCanisters],
) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
    let agent = sns_agent(request)?;
    let fetched = stream::iter(
        targets
            .iter()
            .cloned()
            .map(|sns| fetch_mainnet_sns_metadata_row(&agent, sns)),
    )
    .buffered(SNS_METADATA_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;
    let mut metadata = Vec::with_capacity(fetched.len());
    for row in fetched {
        metadata.push(row?);
    }
    Ok(metadata)
}

async fn fetch_mainnet_sns_metadata_row(
    agent: &Agent,
    sns: MainnetSnsCanisters,
) -> Result<MainnetSnsMetadata, SnsHostError> {
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let (metadata, metadata_error) =
        match fetch_governance_metadata(agent, &governance_canister).await {
            Ok(metadata) => (metadata, None),
            Err(err) => match bounded_query_error_summary(&err) {
                Some(summary) => (GetMetadataResponse::default(), Some(summary)),
                None => return Err(err),
            },
        };
    Ok(mainnet_sns_metadata_from_response(
        sns.root_canister_id,
        metadata,
        metadata_error,
    ))
}

/// Fetch Swap lifecycle evidence for exactly the requested deployed-SNS targets.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_lifecycles(
    request: &SnsSourceRequest,
    targets: &[MainnetSnsCanisters],
) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    block_on_sns(fetch_mainnet_sns_lifecycles_async(request, targets))
}

async fn fetch_mainnet_sns_lifecycles_async(
    request: &SnsSourceRequest,
    targets: &[MainnetSnsCanisters],
) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
    let agent = sns_agent(request)?;
    let fetched = stream::iter(
        targets
            .iter()
            .cloned()
            .map(|sns| fetch_mainnet_sns_lifecycle_row(&agent, sns)),
    )
    .buffered(SNS_METADATA_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;
    let mut lifecycles = Vec::with_capacity(fetched.len());
    for row in fetched {
        lifecycles.push(row?);
    }
    Ok(lifecycles)
}

async fn fetch_mainnet_sns_lifecycle_row(
    agent: &Agent,
    sns: MainnetSnsCanisters,
) -> Result<MainnetSnsLifecycle, SnsHostError> {
    let swap_canister = principal_from_text(&sns.swap_canister_id, "swap_canister_id")?;
    let result = query_canister::<_, GetLifecycleResponse>(
        agent,
        &swap_canister,
        SnsCanisterMethod::GetLifecycle.as_str(),
        "SnsSwapQueryRequest",
        "GetLifecycleResponse",
        &SnsSwapQueryRequest {},
    )
    .await;
    let (lifecycle, lifecycle_error) = match result {
        Ok(response) => response.lifecycle.map_or_else(
            || {
                (
                    None,
                    Some("get_lifecycle: missing lifecycle value".to_string()),
                )
            },
            |lifecycle| (Some(lifecycle), None),
        ),
        Err(error) => match bounded_query_error_summary(&error) {
            Some(summary) => (None, Some(summary)),
            None => return Err(error),
        },
    };
    Ok(MainnetSnsLifecycle {
        root_canister_id: sns.root_canister_id,
        lifecycle,
        lifecycle_name: sns_swap_lifecycle_name(lifecycle).map(str::to_string),
        lifecycle_error,
    })
}

async fn fetch_governance_metadata(
    agent: &Agent,
    governance_canister: &Principal,
) -> Result<GetMetadataResponse, SnsHostError> {
    query_canister(
        agent,
        governance_canister,
        SnsCanisterMethod::GetMetadata.as_str(),
        "GetMetadataRequest",
        "GetMetadataResponse",
        &GetMetadataRequest {},
    )
    .await
}
