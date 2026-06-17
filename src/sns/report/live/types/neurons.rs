//! Module: sns::report::live::types::neurons
//!
//! Responsibility: SNS governance neuron Candid wire types.
//! Does not own: live transport, neuron conversion, cache IO, or rendering.
//! Boundary: mirrors list_neurons request and response payloads.

use super::super::SnsNeuronId;
use candid::{CandidType, Deserialize, Principal};

///
/// ListNeuronsRequest
///
/// Candid request for bounded SNS governance neuron listings.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListNeuronsRequest {
    pub(in crate::sns::report::live) of_principal: Option<Principal>,
    pub(in crate::sns::report::live) limit: u32,
    pub(in crate::sns::report::live) start_page_at: Option<SnsNeuronId>,
}

///
/// ListNeuronsResponse
///
/// Candid response containing SNS governance neuron rows.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListNeuronsResponse {
    pub(in crate::sns::report::live) neurons: Vec<SnsGovernanceNeuron>,
}

///
/// SnsGovernanceNeuron
///
/// Candid SNS governance neuron row converted into report data.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceNeuron {
    pub(in crate::sns::report::live) id: Option<SnsNeuronId>,
    pub(in crate::sns::report::live) staked_maturity_e8s_equivalent: Option<u64>,
    pub(in crate::sns::report::live) maturity_e8s_equivalent: u64,
    pub(in crate::sns::report::live) cached_neuron_stake_e8s: u64,
    pub(in crate::sns::report::live) created_timestamp_seconds: u64,
}
