//! Module: sns::report::live::fetch::reward
//!
//! Responsibility: fetch native reward events and strict full-evidence neuron pages.
//! Does not own: target discovery, bracket ordering, checkpoint assembly, or rendering.
//! Boundary: performs one bounded Governance query per source method invocation.

use super::governance_canister;
use crate::candid_call::query_canister;
use crate::report::CollectionSourceRequest;
use crate::sns::report::{
    MainnetSns, MainnetSnsRewardNeuronPage, SNS_REWARD_CHECKPOINT_PAGE_SIZE, SnsCanisterMethod,
    SnsHostError, SnsNeuronId, SnsRewardEvent,
    live::{
        convert::sns_reward_checkpoint_row,
        query::sns_agent,
        types::{ListNeuronsRequest, ListRewardNeuronsResponse},
    },
    source::validate_mainnet_sns_reward_neuron_page,
};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_reward_event(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsRewardEvent, SnsHostError> {
    let agent = sns_agent(source, request)?;
    let governance_canister = governance_canister(sns)?;
    let method = SnsCanisterMethod::GetLatestRewardEvent.as_str();
    query_canister::<_, _, SnsHostError>(
        agent,
        &governance_canister,
        method,
        method,
        "SnsRewardEvent",
        &(),
    )
    .await
}

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_reward_neuron_page(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
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
    let agent = sns_agent(source, request)?;
    let governance_canister = governance_canister(sns)?;
    let response: ListRewardNeuronsResponse = query_canister::<_, _, SnsHostError>(
        agent,
        &governance_canister,
        SnsCanisterMethod::ListNeurons.as_str(),
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
