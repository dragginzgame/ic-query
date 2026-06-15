mod page;

use super::block_on_sns;
use crate::sns::report::{
    SnsHostError,
    source::{MainnetSns, MainnetSnsNeuronPage, MainnetSnsNeurons, SnsFetchRequest, SnsNeuronId},
};
use page::fetch_mainnet_sns_neuron_page_async;

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
