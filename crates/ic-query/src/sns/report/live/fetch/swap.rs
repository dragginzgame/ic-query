//! Module: sns::report::live::fetch::swap
//!
//! Responsibility: fetch bounded SNS swap lifecycle and sale state.
//! Does not own: SNS lookup, source validation, report assembly, cache IO, or rendering.
//! Boundary: attempts exactly three small public queries and retains component failures as gaps.

use super::block_on_sns;
use crate::sns::report::{
    MainnetSns, MainnetSnsSwap, SnsHostError, SnsSourceRequest, SnsSwapComponent, SnsSwapQueryGap,
    live::{
        convert::{sns_swap_derived_state, sns_swap_lifecycle, sns_swap_sale_parameters},
        query::{principal_from_text, query_canister, sns_agent},
        types::{
            GetDerivedStateResponse, GetLifecycleResponse, GetSaleParametersResponse,
            SnsSwapQueryRequest,
        },
    },
    source::{
        SNS_SWAP_DERIVED_STATE_METHOD, SNS_SWAP_LIFECYCLE_METHOD, SNS_SWAP_SALE_PARAMETERS_METHOD,
        sns_swap_component_method,
    },
};

/// Fetch the three bounded native state components for one resolved SNS swap.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_swap(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsSwap, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_swap_async(request, sns))
}

async fn fetch_mainnet_sns_swap_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsSwap, SnsHostError> {
    let agent = sns_agent(request)?;
    let swap_canister = principal_from_text(&sns.swap_canister_id, "swap_canister_id")?;
    let request = SnsSwapQueryRequest {};
    let mut gaps = Vec::new();

    let lifecycle = match query_canister::<_, GetLifecycleResponse>(
        &agent,
        &swap_canister,
        SNS_SWAP_LIFECYCLE_METHOD,
        "SnsSwapQueryRequest",
        "GetLifecycleResponse",
        &request,
    )
    .await
    {
        Ok(response) => Some(sns_swap_lifecycle(response)),
        Err(error) => {
            gaps.push(query_gap(SnsSwapComponent::Lifecycle, error));
            None
        }
    };
    let sale_parameters = match query_canister::<_, GetSaleParametersResponse>(
        &agent,
        &swap_canister,
        SNS_SWAP_SALE_PARAMETERS_METHOD,
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
    let derived_state = match query_canister::<_, GetDerivedStateResponse>(
        &agent,
        &swap_canister,
        SNS_SWAP_DERIVED_STATE_METHOD,
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
        swap_canister_id: sns.swap_canister_id.clone(),
        lifecycle_method: SNS_SWAP_LIFECYCLE_METHOD.to_string(),
        sale_parameters_method: SNS_SWAP_SALE_PARAMETERS_METHOD.to_string(),
        derived_state_method: SNS_SWAP_DERIVED_STATE_METHOD.to_string(),
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
        method: sns_swap_component_method(component).to_string(),
        reason: error.to_string(),
    }
}
