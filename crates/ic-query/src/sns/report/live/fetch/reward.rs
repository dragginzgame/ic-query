//! Module: sns::report::live::fetch::reward
//!
//! Responsibility: fetch native reward events and strict full-evidence neuron pages.
//! Does not own: target discovery, bracket ordering, checkpoint assembly, or rendering.
//! Boundary: performs one bounded Governance query per source method invocation.

use super::{block_on_sns, governance_canister};
use crate::sns::report::{
    MainnetSns, MainnetSnsRewardNeuronPage, SNS_REWARD_CHECKPOINT_PAGE_SIZE, SnsHostError,
    SnsNeuronId, SnsRewardEvent, SnsSourceRequest,
    live::{
        convert::sns_reward_checkpoint_row,
        query::{query_canister, sns_agent},
        types::{ListNeuronsRequest, ListRewardNeuronsResponse},
    },
    source::validate_mainnet_sns_reward_neuron_page,
};

const REWARD_EVENT_METHOD: &str = "get_latest_reward_event";

/// Fetch one complete native latest reward event.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_reward_event(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsRewardEvent, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_reward_event_async(request, sns))
}

/// Fetch one strict reward-checkpoint neuron page.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_reward_neuron_page(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
) -> Result<MainnetSnsRewardNeuronPage, SnsHostError> {
    if limit != SNS_REWARD_CHECKPOINT_PAGE_SIZE {
        return Err(SnsHostError::InvalidSourceData {
            capability: "SNS reward checkpoint",
            reason: format!(
                "live reward pages require fixed limit {SNS_REWARD_CHECKPOINT_PAGE_SIZE}, got {limit}"
            ),
        });
    }
    block_on_sns(fetch_mainnet_sns_reward_neuron_page_async(
        request,
        sns,
        start_page_at,
    ))
}

async fn fetch_mainnet_sns_reward_event_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsRewardEvent, SnsHostError> {
    let agent = sns_agent(request)?;
    let governance_canister = governance_canister(sns)?;
    query_canister(
        &agent,
        &governance_canister,
        REWARD_EVENT_METHOD,
        "get_latest_reward_event",
        "SnsRewardEvent",
        &(),
    )
    .await
}

async fn fetch_mainnet_sns_reward_neuron_page_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    start_page_at: Option<&SnsNeuronId>,
) -> Result<MainnetSnsRewardNeuronPage, SnsHostError> {
    let agent = sns_agent(request)?;
    let governance_canister = governance_canister(sns)?;
    let response: ListRewardNeuronsResponse = query_canister(
        &agent,
        &governance_canister,
        "list_neurons",
        "ListNeuronsRequest",
        "ListRewardNeuronsResponse",
        &ListNeuronsRequest {
            of_principal: None,
            limit: SNS_REWARD_CHECKPOINT_PAGE_SIZE,
            start_page_at: start_page_at.cloned(),
        },
    )
    .await?;
    let next_cursor = (response.neurons.len() == SNS_REWARD_CHECKPOINT_PAGE_SIZE as usize)
        .then(|| response.neurons.last().and_then(|neuron| neuron.id.clone()))
        .flatten();
    let page = MainnetSnsRewardNeuronPage {
        neurons: response
            .neurons
            .into_iter()
            .map(sns_reward_checkpoint_row)
            .collect::<Result<Vec<_>, _>>()?,
        next_cursor,
    };
    validate_mainnet_sns_reward_neuron_page(&page)?;
    Ok(page)
}
