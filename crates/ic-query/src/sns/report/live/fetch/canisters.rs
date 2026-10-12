//! Module: sns::report::live::fetch::canisters
//!
//! Responsibility: fetch SNS Root canister inventory and operational health.
//! Does not own: SNS lookup, report assembly, cache IO, or rendering.
//! Boundary: performs one query plus one explicitly read-only ingress call.
use crate::sns::report::{
    MainnetSns, MainnetSnsCanisterInventory, SnsCanisterMethod, SnsHostError, SnsSourceRequest,
    live::{
        convert::{mainnet_sns_canister_inventory, mainnet_sns_canister_inventory_without_health},
        query::{principal_from_text, query_canister, sns_agent, update_canister},
        types::{
            GetSnsCanistersSummaryRequest, GetSnsCanistersSummaryResponse, ListSnsCanistersRequest,
            ListSnsCanistersResponse,
        },
    },
};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_canisters(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let root_canister = principal_from_text(&sns.root_canister_id, "root_canister_id")?;
    let inventory: ListSnsCanistersResponse = query_canister(
        agent,
        &root_canister,
        SnsCanisterMethod::ListSnsCanisters.as_str(),
        "ListSnsCanistersRequest",
        "ListSnsCanistersResponse",
        &ListSnsCanistersRequest {},
    )
    .await?;
    let health_request = GetSnsCanistersSummaryRequest::read_only();
    let health = update_canister::<_, GetSnsCanistersSummaryResponse>(
        agent,
        &root_canister,
        SnsCanisterMethod::GetSnsCanistersSummary.as_str(),
        "GetSnsCanistersSummaryRequest",
        "GetSnsCanistersSummaryResponse",
        &health_request,
    )
    .await;
    match health {
        Ok(health) => mainnet_sns_canister_inventory(inventory, health),
        Err(error) => mainnet_sns_canister_inventory_without_health(inventory, error.to_string()),
    }
}
