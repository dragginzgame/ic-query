use super::super::governance_canister;
use crate::sns::report::{
    SnsHostError,
    live::{
        convert::{sns_neuron_cursor, sns_neuron_row},
        query::{principal_from_text, query_canister, sns_agent},
        types::{ListNeuronsRequest, ListNeuronsResponse},
    },
    source::{MainnetSns, MainnetSnsNeuronPage, SnsFetchRequest, SnsNeuronId},
};

pub(super) async fn fetch_mainnet_sns_neuron_page_async(
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
