use super::super::SnsNeuronId;
use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListNeuronsRequest {
    pub(in crate::sns::report::live) of_principal: Option<Principal>,
    pub(in crate::sns::report::live) limit: u32,
    pub(in crate::sns::report::live) start_page_at: Option<SnsNeuronId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListNeuronsResponse {
    pub(in crate::sns::report::live) neurons: Vec<SnsGovernanceNeuron>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceNeuron {
    pub(in crate::sns::report::live) id: Option<SnsNeuronId>,
    pub(in crate::sns::report::live) staked_maturity_e8s_equivalent: Option<u64>,
    pub(in crate::sns::report::live) maturity_e8s_equivalent: u64,
    pub(in crate::sns::report::live) cached_neuron_stake_e8s: u64,
    pub(in crate::sns::report::live) created_timestamp_seconds: u64,
}
