//! Module: sns::report::live::fetch::swap
//!
//! Responsibility: fetch bounded SNS swap lifecycle and sale state.
//! Does not own: SNS lookup, source validation, report assembly, cache IO, or rendering.
//! Boundary: attempts exactly three small public queries and retains component failures as gaps.
use crate::candid_call::query_canister;
use crate::sns::report::{
    MainnetSns, MainnetSnsSwap, SnsCanisterMethod, SnsHostError, SnsSourceRequest,
    SnsSwapComponent, SnsSwapQueryGap,
    live::{
        convert::{sns_swap_derived_state, sns_swap_lifecycle, sns_swap_sale_parameters},
        query::{principal_from_text, sns_agent},
        types::{
            GetDerivedStateResponse, GetLifecycleResponse, GetSaleParametersResponse,
            SnsSwapQueryRequest,
        },
    },
    source::sns_swap_component_method,
};
use ic_platform::{agent::Agent, candid::contract::Principal};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_swap(
    source: &crate::sns::LiveSnsSource,
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsSwap, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let swap_canister = principal_from_text(&sns.canisters.swap_canister_id, "swap_canister_id")?;
    let request = SnsSwapQueryRequest {};
    let mut gaps = Vec::new();

    let lifecycle = match query_lifecycle(agent, &swap_canister).await {
        Ok(response) => Some(sns_swap_lifecycle(response)),
        Err(error) => {
            gaps.push(query_gap(SnsSwapComponent::Lifecycle, error));
            None
        }
    };
    let sale_parameters = match query_canister::<_, GetSaleParametersResponse, SnsHostError>(
        agent,
        &swap_canister,
        SnsCanisterMethod::GetSaleParameters.as_str(),
        "SnsSwapQueryRequest",
        "GetSaleParametersResponse",
        &request,
    )
    .await
    {
        Ok(response) => sns_swap_sale_parameters(response),
        Err(error) => {
            gaps.push(query_gap(SnsSwapComponent::SaleParameters, error));
            None
        }
    };
    let derived_state = match query_canister::<_, GetDerivedStateResponse, SnsHostError>(
        agent,
        &swap_canister,
        SnsCanisterMethod::GetDerivedState.as_str(),
        "SnsSwapQueryRequest",
        "GetDerivedStateResponse",
        &request,
    )
    .await
    {
        Ok(response) => Some(sns_swap_derived_state(response)),
        Err(error) => {
            gaps.push(query_gap(SnsSwapComponent::DerivedState, error));
            None
        }
    };

    Ok(MainnetSnsSwap {
        swap_canister_id: sns.canisters.swap_canister_id.clone(),
        lifecycle_method: SnsCanisterMethod::GetLifecycle,
        sale_parameters_method: SnsCanisterMethod::GetSaleParameters,
        derived_state_method: SnsCanisterMethod::GetDerivedState,
        point_in_time_guaranteed: false,
        lifecycle,
        sale_parameters,
        derived_state,
        gaps,
    })
}

fn query_gap(component: SnsSwapComponent, error: SnsHostError) -> SnsSwapQueryGap {
    SnsSwapQueryGap {
        component,
        method: sns_swap_component_method(component),
        reason: error.to_string(),
    }
}

pub(super) async fn query_lifecycle(
    agent: &Agent,
    swap_canister: &Principal,
) -> Result<GetLifecycleResponse, SnsHostError> {
    query_canister(
        agent,
        swap_canister,
        SnsCanisterMethod::GetLifecycle.as_str(),
        "SnsSwapQueryRequest",
        "GetLifecycleResponse",
        &SnsSwapQueryRequest {},
    )
    .await
}
