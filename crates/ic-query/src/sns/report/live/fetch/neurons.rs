//! Module: sns::report::live::fetch::neurons
//!
//! Responsibility: fetch SNS governance neurons.
//! Does not own: lookup resolution, cache storage, sorting, or rendering.
//! Boundary: queries bounded and paged neuron data for live reports and refreshes.

use super::block_on_sns;
use crate::sns::report::{
    SnsCanisterMethod, SnsHostError,
    live::{
        convert::{mainnet_sns_neuron, sns_neuron_row},
        fetch::governance_canister,
        query::{principal_from_text, query_canister, sns_agent},
        types::{
            GetNeuronRequest, GetNeuronResponse, GetNeuronResult, ListNeuronsRequest,
            ListNeuronsResponse,
        },
    },
    source::{
        MainnetSns, MainnetSnsNeuron, MainnetSnsNeuronPage, MainnetSnsNeurons, SnsNeuronId,
        SnsSourceRequest, sns_neuron_id_from_text, validate_mainnet_sns_neuron,
        validate_mainnet_sns_neuron_page,
    },
};

/// Fetch one exact full SNS neuron for one resolved mainnet SNS.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_neuron(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    neuron_id: &str,
) -> Result<MainnetSnsNeuron, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_neuron_async(request, sns, neuron_id))
}

/// Fetch a bounded SNS neuron listing for one resolved mainnet SNS.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_neurons(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    limit: u32,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeurons, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_neurons_async(
        request,
        sns,
        limit,
        owner_principal_id,
    ))
}

/// Fetch one SNS neuron page for complete snapshot refresh.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_neuron_page(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_neuron_page_async(
        request,
        sns,
        limit,
        start_page_at,
        owner_principal_id,
    ))
}

async fn fetch_mainnet_sns_neurons_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    limit: u32,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeurons, SnsHostError> {
    let page =
        fetch_mainnet_sns_neuron_page_async(request, sns, limit, None, owner_principal_id).await?;
    Ok(MainnetSnsNeurons {
        neurons: page.neurons,
    })
}

async fn fetch_mainnet_sns_neuron_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    neuron_id: &str,
) -> Result<MainnetSnsNeuron, SnsHostError> {
    let parsed_neuron_id = sns_neuron_id_from_text(neuron_id)?;
    let agent = sns_agent(request)?;
    let governance_canister = governance_canister(sns)?;
    let method = SnsCanisterMethod::GetNeuron.as_str();
    let response: GetNeuronResponse = query_canister(
        &agent,
        &governance_canister,
        method,
        "GetNeuronRequest",
        "GetNeuronResponse",
        &GetNeuronRequest {
            neuron_id: Some(parsed_neuron_id),
        },
    )
    .await?;
    let neuron = match response
        .result
        .ok_or(SnsHostError::MissingGovernanceResult { method })?
    {
        GetNeuronResult::Neuron(neuron) => mainnet_sns_neuron(*neuron)?,
        GetNeuronResult::Error(error) => {
            return Err(SnsHostError::GovernanceError {
                method,
                error_type: error.error_type,
                message: error.error_message,
            });
        }
    };
    validate_mainnet_sns_neuron(&neuron, neuron_id)?;
    Ok(neuron)
}

async fn fetch_mainnet_sns_neuron_page_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    let agent = sns_agent(request)?;
    let governance_canister = governance_canister(sns)?;
    let owner_principal = owner_principal_id
        .map(|principal| principal_from_text(principal, "owner_principal_id"))
        .transpose()?;
    let response: ListNeuronsResponse = query_canister(
        &agent,
        &governance_canister,
        SnsCanisterMethod::ListNeurons.as_str(),
        "ListNeuronsRequest",
        "ListNeuronsResponse",
        &ListNeuronsRequest {
            of_principal: owner_principal,
            limit,
            start_page_at: start_page_at.cloned(),
        },
    )
    .await?;
    let last_cursor = response.neurons.last().and_then(|neuron| neuron.id.clone());
    let neurons = response
        .neurons
        .into_iter()
        .map(sns_neuron_row)
        .collect::<Result<Vec<_>, _>>()?;
    let page = MainnetSnsNeuronPage {
        neurons,
        last_cursor,
    };
    validate_mainnet_sns_neuron_page(&page, limit)?;
    Ok(page)
}
