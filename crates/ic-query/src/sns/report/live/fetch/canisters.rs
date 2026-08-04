//! Module: sns::report::live::fetch::canisters
//!
//! Responsibility: fetch SNS Root canister inventory and operational health.
//! Does not own: SNS lookup, report assembly, cache IO, or rendering.
//! Boundary: performs one query plus one explicitly read-only ingress call.

use super::block_on_sns;
use crate::sns::report::{
    MainnetSns, MainnetSnsCanisterInventory, SnsCanisterMethod, SnsHostError, SnsSourceRequest,
    live::{
        convert::mainnet_sns_canister_inventory,
        query::{principal_from_text, query_canister, sns_agent, update_canister},
        types::{
            GetSnsCanistersSummaryRequest, GetSnsCanistersSummaryResponse, ListSnsCanistersRequest,
            ListSnsCanistersResponse,
        },
    },
};

/// Fetch current Root inventory and health without asking Root to update state.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_canisters(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_canisters_async(request, sns))
}

async fn fetch_mainnet_sns_canisters_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
    let agent = sns_agent(request)?;
    let root_canister = principal_from_text(&sns.root_canister_id, "root_canister_id")?;
    let inventory: ListSnsCanistersResponse = query_canister(
        &agent,
        &root_canister,
        SnsCanisterMethod::ListSnsCanisters.as_str(),
        "ListSnsCanistersRequest",
        "ListSnsCanistersResponse",
        &ListSnsCanistersRequest {},
    )
    .await?;
    let health_request = GetSnsCanistersSummaryRequest::read_only();
    let health: GetSnsCanistersSummaryResponse = update_canister(
        &agent,
        &root_canister,
        SnsCanisterMethod::GetSnsCanistersSummary.as_str(),
        "GetSnsCanistersSummaryRequest",
        "GetSnsCanistersSummaryResponse",
        &health_request,
    )
    .await?;
    mainnet_sns_canister_inventory(inventory, health)
}
