//! Module: sns::report::live::types::neurons
//!
//! Responsibility: SNS governance neuron Candid wire types.
//! Does not own: live transport, neuron conversion, cache IO, or rendering.
//! Boundary: mirrors list_neurons request and response payloads.

use super::SnsGovernanceError;
use crate::sns::report::SnsNeuronId;
use ic_platform::candid::contract::{CandidType, Principal, serde::Deserialize};

///
/// GetNeuronRequest
///
/// Candid request for one exact SNS Governance neuron.
///

#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) struct GetNeuronRequest {
    pub(in crate::sns::report::live) neuron_id: Option<SnsNeuronId>,
}

///
/// GetNeuronResponse
///
/// Candid response containing one exact neuron or a Governance error.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) struct GetNeuronResponse {
    pub(in crate::sns::report::live) result: Option<GetNeuronResult>,
}

///
/// GetNeuronResult
///
/// Native result variant returned by exact SNS neuron lookup.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) enum GetNeuronResult {
    Error(SnsGovernanceError),
    Neuron(Box<SnsGovernanceNeuron>),
}

///
/// ListNeuronsRequest
///
/// Candid request for bounded SNS governance neuron listings.
///

#[derive(CandidType)]
#[candid_path("ic_platform::candid::contract")]
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
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) struct ListNeuronsResponse {
    pub(in crate::sns::report::live) neurons: Vec<SnsGovernanceNeuron>,
}

///
/// SnsGovernanceNeuron
///
/// Candid SNS governance neuron row converted into report data.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) struct SnsGovernanceNeuron {
    pub(in crate::sns::report::live) permissions: Vec<SnsGovernanceNeuronPermission>,
    pub(in crate::sns::report::live) id: Option<SnsNeuronId>,
    pub(in crate::sns::report::live) staked_maturity_e8s_equivalent: Option<u64>,
    pub(in crate::sns::report::live) maturity_e8s_equivalent: u64,
    pub(in crate::sns::report::live) cached_neuron_stake_e8s: u64,
    pub(in crate::sns::report::live) created_timestamp_seconds: u64,
    pub(in crate::sns::report::live) source_nns_neuron_id: Option<u64>,
    pub(in crate::sns::report::live) auto_stake_maturity: Option<bool>,
    pub(in crate::sns::report::live) aging_since_timestamp_seconds: u64,
    pub(in crate::sns::report::live) dissolve_state: Option<SnsGovernanceDissolveState>,
    pub(in crate::sns::report::live) voting_power_percentage_multiplier: u64,
    pub(in crate::sns::report::live) vesting_period_seconds: Option<u64>,
    pub(in crate::sns::report::live) neuron_fees_e8s: u64,
}

///
/// SnsGovernanceNeuronPermission
///
/// Native principal permission entry embedded in one neuron.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) struct SnsGovernanceNeuronPermission {
    pub(in crate::sns::report::live) principal: Option<Principal>,
    pub(in crate::sns::report::live) permission_type: Vec<i32>,
}

///
/// SnsGovernanceDissolveState
///
/// Candid dissolve-state alternative returned by SNS governance.
///

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::sns::report::live) enum SnsGovernanceDissolveState {
    DissolveDelaySeconds(u64),
    WhenDissolvedTimestampSeconds(u64),
}
