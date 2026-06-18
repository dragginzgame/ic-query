//! Module: sns::report::source::traits::neurons
//!
//! Responsibility: SNS neuron source contract.
//! Does not own: live governance transport, cache reads, sorting, or rendering.
//! Boundary: extends deployed SNS lookup sources with bounded and paged neuron fetching.

use super::list::SnsListSource;
use crate::sns::report::{
    MainnetSns, MainnetSnsNeuronPage, MainnetSnsNeurons, SnsFetchRequest, SnsHostError, SnsNeuronId,
};

///
/// SnsNeuronsSource
///
/// Source contract for fetching bounded and paged SNS neuron data.
///

pub(in crate::sns::report) trait SnsNeuronsSource: SnsListSource {
    /// Fetch a bounded SNS neuron listing for one resolved SNS.
    fn fetch_sns_neurons(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError>;

    /// Fetch one SNS neuron page for complete snapshot refresh.
    fn fetch_sns_neuron_page(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError>;
}
