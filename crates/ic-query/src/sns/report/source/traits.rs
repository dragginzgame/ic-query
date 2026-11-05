//! Read-only neuron capabilities for explicitly addressed Governance canisters.
use super::super::{MainnetSnsNeuronPage, SnsHostError, SnsNeuronId, SnsNeuronRow};
use crate::report::CollectionSourceRequest;

///
/// SnsNeuronSource
///
/// Source contract for fetching one exact full SNS neuron detail.
///

pub trait SnsNeuronSource: Sync {
    /// Fetch exactly one full native SNS Governance neuron.
    fn fetch_sns_neuron<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        governance_canister_id: &'a str,
        neuron_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<SnsNeuronRow, SnsHostError>>;
}

///
/// SnsNeuronsSource
///
/// Source contract for fetching bounded and paged SNS neuron data.
///

pub trait SnsNeuronsSource: Sync {
    /// Fetch a bounded SNS neuron listing for one resolved SNS.
    fn fetch_sns_neurons<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        governance_canister_id: &'a str,
        limit: u32,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsNeuronRow>, SnsHostError>>;

    /// Fetch one ascending neuron page for complete snapshot refresh.
    /// A supplied cursor must equal the final row's id; empty pages omit it.
    /// A repeated boundary row is allowed; full pages require an advancing cursor.
    /// Short terminal pages may omit the cursor.
    fn fetch_sns_neuron_page<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        governance_canister_id: &'a str,
        limit: u32,
        start_page_at: Option<&'a SnsNeuronId>,
        owner_principal_id: Option<&'a str>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsNeuronPage, SnsHostError>>;
}
