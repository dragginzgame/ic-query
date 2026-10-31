//! Module: sns::report::live::fetch::list
//!
//! Responsibility: fetch deployed SNS inventory and explicitly targeted catalog enrichment.
//! Does not own: report assembly, command parsing, cache IO, or rendering.
//! Boundary: keeps SNS-W inventory separate from bounded Governance metadata and Swap lifecycle queries.
use crate::candid_call::query_canister;
use crate::report::CollectionSourceRequest;
use crate::sns::report::live::{
    convert::{mainnet_sns_canisters_from_deployed_sns, mainnet_sns_metadata_from_response},
    query::{principal_from_text, sns_agent},
    types::{
        GetMetadataRequest, GetMetadataResponse, ListDeployedSnsesRequest,
        ListDeployedSnsesResponse,
    },
};
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SNS_METADATA_CONCURRENCY, SnsCanisterMethod, SnsHostError,
    enforce_mainnet_network,
    source::{MainnetSnsInventory, MainnetSnsLifecycle, MainnetSnsMetadata},
};
use crate::subnet_catalog::MAINNET_NETWORK;
use futures_util::{StreamExt, stream};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;
use std::future::Future;

use crate::sns::report::MainnetSnsCanisters;
pub(in crate::sns::report::live) async fn fetch_mainnet_sns_inventory(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
) -> Result<MainnetSnsInventory, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let sns_wasm_canister =
        principal_from_text(MAINNET_SNS_WASM_CANISTER_ID, "sns_wasm_canister_id")?;
    let response: ListDeployedSnsesResponse = query_canister::<_, _, SnsHostError>(
        agent,
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
    request: &CollectionSourceRequest,
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

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_metadata(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
    targets: &[MainnetSnsCanisters],
) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let agent = sns_agent(source, request)?;
    // Index tasks to keep borrowed-target lifetime inference compatible with the Send source future.
    collect_enrichment_rows(
        (0..targets.len()).map(|index| fetch_mainnet_sns_metadata_row(agent, &targets[index])),
    )
    .await
}

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_metadata_row(
    agent: &Agent,
    sns: &MainnetSnsCanisters,
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
        sns.root_canister_id.clone(),
        metadata,
        metadata_error,
    ))
}

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_lifecycles(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
    targets: &[MainnetSnsCanisters],
) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let agent = sns_agent(source, request)?;
    // Index tasks to keep borrowed-target lifetime inference compatible with the Send source future.
    collect_enrichment_rows(
        (0..targets.len()).map(|index| fetch_mainnet_sns_lifecycle_row(agent, &targets[index])),
    )
    .await
}

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_lifecycle_row(
    agent: &Agent,
    sns: &MainnetSnsCanisters,
) -> Result<MainnetSnsLifecycle, SnsHostError> {
    let swap_canister = principal_from_text(&sns.swap_canister_id, "swap_canister_id")?;
    let result = super::swap::query_lifecycle(agent, &swap_canister).await;
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
        root_canister_id: sns.root_canister_id.clone(),
        lifecycle,

        lifecycle_error,
    })
}

async fn fetch_governance_metadata(
    agent: &Agent,
    governance_canister: &Principal,
) -> Result<GetMetadataResponse, SnsHostError> {
    query_canister::<_, _, SnsHostError>(
        agent,
        governance_canister,
        SnsCanisterMethod::GetMetadata.as_str(),
        "GetMetadataRequest",
        "GetMetadataResponse",
        &GetMetadataRequest {},
    )
    .await
}

async fn collect_enrichment_rows<Row, Task>(
    tasks: impl IntoIterator<Item = Task>,
) -> Result<Vec<Row>, SnsHostError>
where
    Task: Future<Output = Result<Row, SnsHostError>>,
{
    stream::iter(tasks)
        .buffered(SNS_METADATA_CONCURRENCY)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}

/// Return a compact bounded-query error summary when the error is displayable.
fn bounded_query_error_summary(err: &SnsHostError) -> Option<String> {
    let summary = match err {
        SnsHostError::AgentCall { method, reason } => Some(format!("{method}: {reason}")),
        SnsHostError::CandidEncode { message, reason } => {
            Some(format!("encode {message}: {reason}"))
        }
        SnsHostError::CandidDecode { message, reason } => {
            Some(format!("decode {message}: {reason}"))
        }
        SnsHostError::GovernanceError {
            method,
            error_type,
            message,
        } => Some(format!("{method} governance error {error_type}: {message}")),
        SnsHostError::MissingGovernanceResult { method } => {
            Some(format!("{method}: missing governance result"))
        }
        _ => None,
    }?;
    let summary = summary.trim();
    (!summary.is_empty()).then(|| summary.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[tokio::test]
    async fn enrichment_bounds_concurrency_preserves_order_and_finishes_before_error() {
        let count = SNS_METADATA_CONCURRENCY * 2;
        for fail_first in [false, true] {
            let active = Cell::new(0);
            let peak = Cell::new(0);
            let completed = Cell::new(0);
            let result = collect_enrichment_rows((0..count).map(|index| {
                let active = &active;
                let peak = &peak;
                let completed = &completed;
                async move {
                    active.set(active.get() + 1);
                    peak.set(peak.get().max(active.get()));
                    tokio::task::yield_now().await;
                    active.set(active.get() - 1);
                    completed.set(completed.get() + 1);
                    if fail_first && index == 0 {
                        Err(SnsHostError::InvalidSourceData {
                            capability: "fixture",
                            reason: "first row failed".to_string(),
                        })
                    } else {
                        Ok(index)
                    }
                }
            }))
            .await;
            assert_eq!(completed.get(), count);
            assert_eq!(peak.get(), SNS_METADATA_CONCURRENCY);
            assert_eq!(active.get(), 0);
            if fail_first {
                assert!(matches!(
                    result,
                    Err(SnsHostError::InvalidSourceData {
                        capability: "fixture",
                        ..
                    })
                ));
            } else {
                assert_eq!(result.unwrap(), (0..count).collect::<Vec<_>>());
            }
        }
    }
}
