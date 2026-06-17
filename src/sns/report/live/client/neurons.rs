//! Module: sns::report::live::client::neurons
//!
//! Responsibility: live SNS neuron source implementation.
//! Does not own: governance query construction, cache reads, or rendering.
//! Boundary: delegates neuron source trait methods to live fetch helpers.

use super::LiveSnsSource;
use crate::sns::report::{
    MainnetSns, MainnetSnsNeuronPage, MainnetSnsNeurons, SnsFetchRequest, SnsHostError,
    SnsNeuronId, SnsNeuronsSource,
    live::fetch::{fetch_mainnet_sns_neuron_page, fetch_mainnet_sns_neurons},
};

impl SnsNeuronsSource for LiveSnsSource {
    fn fetch_sns_neurons(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        fetch_mainnet_sns_neurons(request, sns, limit, owner_principal_id)
    }

    fn fetch_sns_neuron_page(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        fetch_mainnet_sns_neuron_page(request, sns, limit, start_page_at, owner_principal_id)
    }
}
