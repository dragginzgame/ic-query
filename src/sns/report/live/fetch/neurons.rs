use super::{block_on_sns, governance_canister};
use crate::sns::report::{
    SnsHostError,
    live::{
        convert::{sns_neuron_cursor, sns_neuron_row},
        query::{principal_from_text, query_canister, sns_agent},
        types::{ListNeuronsRequest, ListNeuronsResponse},
    },
    source::{MainnetSns, MainnetSnsNeuronPage, MainnetSnsNeurons, SnsFetchRequest, SnsNeuronId},
};

pub(in crate::sns::report::live) fn fetch_mainnet_sns_neurons(
    request: &SnsFetchRequest,
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

pub(in crate::sns::report::live) fn fetch_mainnet_sns_neuron_page(
    request: &SnsFetchRequest,
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
    request: &SnsFetchRequest,
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

async fn fetch_mainnet_sns_neuron_page_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister = governance_canister(sns)?;
    let owner_principal = owner_principal_id
        .map(|principal| principal_from_text(principal, "owner_principal_id"))
        .transpose()?;
    let response: ListNeuronsResponse = query_canister(
        &agent,
        &governance_canister,
        "list_neurons",
        "ListNeuronsRequest",
        "ListNeuronsResponse",
        &ListNeuronsRequest {
            of_principal: owner_principal,
            limit,
            start_page_at: start_page_at.cloned(),
        },
    )
    .await?;
    let last_cursor = response.neurons.iter().rev().find_map(sns_neuron_cursor);
    Ok(MainnetSnsNeuronPage {
        neurons: response.neurons.into_iter().map(sns_neuron_row).collect(),
        last_cursor,
    })
}
