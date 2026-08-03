//! Module: sns::report::live::types::neurons
//!
//! Responsibility: SNS governance neuron Candid wire types.
//! Does not own: live transport, neuron conversion, cache IO, or rendering.
//! Boundary: mirrors list_neurons request and response payloads.

use super::{SnsGovernanceError, SnsMetricsAccount, SnsTopic};
use crate::sns::report::SnsNeuronId;
use candid::{CandidType, Deserialize, Principal};

///
/// GetNeuronRequest
///
/// Candid request for one exact SNS Governance neuron.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetNeuronRequest {
    pub(in crate::sns::report::live) neuron_id: Option<SnsNeuronId>,
}

///
/// GetNeuronResponse
///
/// Candid response containing one exact neuron or a Governance error.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetNeuronResponse {
    pub(in crate::sns::report::live) result: Option<GetNeuronResult>,
}

///
/// GetNeuronResult
///
/// Native result variant returned by exact SNS neuron lookup.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum GetNeuronResult {
    Error(SnsGovernanceError),
    Neuron(Box<SnsGovernanceNeuronDetail>),
}

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
/// ListRewardNeuronsResponse
///
/// Candid response projected to the variable fields required by reward checkpoints.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListRewardNeuronsResponse {
    pub(in crate::sns::report::live) neurons: Vec<SnsGovernanceRewardNeuron>,
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
    pub(in crate::sns::report::live) source_nns_neuron_id: Option<u64>,
    pub(in crate::sns::report::live) auto_stake_maturity: Option<bool>,
    pub(in crate::sns::report::live) aging_since_timestamp_seconds: u64,
    pub(in crate::sns::report::live) dissolve_state: Option<SnsGovernanceDissolveState>,
    pub(in crate::sns::report::live) voting_power_percentage_multiplier: u64,
    pub(in crate::sns::report::live) vesting_period_seconds: Option<u64>,
    pub(in crate::sns::report::live) neuron_fees_e8s: u64,
}

///
/// SnsGovernanceNeuronDetail
///
/// Full Candid SNS Governance neuron returned by exact lookup.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceNeuronDetail {
    pub(in crate::sns::report::live) id: Option<SnsNeuronId>,
    pub(in crate::sns::report::live) staked_maturity_e8s_equivalent: Option<u64>,
    pub(in crate::sns::report::live) permissions: Vec<SnsGovernanceNeuronPermission>,
    pub(in crate::sns::report::live) maturity_e8s_equivalent: u64,
    pub(in crate::sns::report::live) cached_neuron_stake_e8s: u64,
    pub(in crate::sns::report::live) created_timestamp_seconds: u64,
    pub(in crate::sns::report::live) source_nns_neuron_id: Option<u64>,
    pub(in crate::sns::report::live) auto_stake_maturity: Option<bool>,
    pub(in crate::sns::report::live) aging_since_timestamp_seconds: u64,
    pub(in crate::sns::report::live) dissolve_state: Option<SnsGovernanceDissolveState>,
    pub(in crate::sns::report::live) voting_power_percentage_multiplier: u64,
    pub(in crate::sns::report::live) vesting_period_seconds: Option<u64>,
    pub(in crate::sns::report::live) disburse_maturity_in_progress:
        Vec<SnsGovernanceMaturityDisbursement>,
    pub(in crate::sns::report::live) followees: Vec<(u64, SnsGovernanceFollowees)>,
    pub(in crate::sns::report::live) topic_followees: Option<SnsGovernanceTopicFollowees>,
    pub(in crate::sns::report::live) neuron_fees_e8s: u64,
}

///
/// SnsGovernanceRewardNeuron
///
/// Native neuron projection retaining only reward-checkpoint evidence.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceRewardNeuron {
    pub(in crate::sns::report::live) id: Option<SnsNeuronId>,
    pub(in crate::sns::report::live) staked_maturity_e8s_equivalent: Option<u64>,
    pub(in crate::sns::report::live) permissions: Vec<SnsGovernanceNeuronPermission>,
    pub(in crate::sns::report::live) maturity_e8s_equivalent: u64,
    pub(in crate::sns::report::live) created_timestamp_seconds: u64,
    pub(in crate::sns::report::live) auto_stake_maturity: Option<bool>,
    pub(in crate::sns::report::live) disburse_maturity_in_progress:
        Vec<SnsGovernanceMaturityDisbursement>,
}

///
/// SnsGovernanceNeuronPermission
///
/// Native principal permission entry embedded in one neuron.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceNeuronPermission {
    pub(in crate::sns::report::live) principal: Option<Principal>,
    pub(in crate::sns::report::live) permission_type: Vec<i32>,
}

///
/// SnsGovernanceMaturityDisbursement
///
/// Native pending maturity disbursement embedded in one neuron.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceMaturityDisbursement {
    pub(in crate::sns::report::live) timestamp_of_disbursement_seconds: u64,
    pub(in crate::sns::report::live) amount_e8s: u64,
    pub(in crate::sns::report::live) account_to_disburse_to: Option<SnsMetricsAccount>,
    pub(in crate::sns::report::live) finalize_disbursement_timestamp_seconds: Option<u64>,
}

///
/// SnsGovernanceFollowees
///
/// Native neuron-id collection used by legacy and topic following.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceFollowees {
    pub(in crate::sns::report::live) followees: Vec<SnsNeuronId>,
}

///
/// SnsGovernanceFollowee
///
/// Native topic followee with an optional neuron id and alias.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceFollowee {
    pub(in crate::sns::report::live) neuron_id: Option<SnsNeuronId>,
    pub(in crate::sns::report::live) alias: Option<String>,
}

///
/// SnsGovernanceFolloweesForTopic
///
/// Native followee collection and topic variant for one topic map entry.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceFolloweesForTopic {
    pub(in crate::sns::report::live) followees: Vec<SnsGovernanceFollowee>,
    pub(in crate::sns::report::live) topic: Option<SnsTopic>,
}

///
/// SnsGovernanceTopicFollowees
///
/// Native optional topic-following map wrapper embedded in one neuron.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceTopicFollowees {
    pub(in crate::sns::report::live) topic_id_to_followees:
        Vec<(i32, SnsGovernanceFolloweesForTopic)>,
}

///
/// SnsGovernanceDissolveState
///
/// Candid dissolve-state alternative returned by SNS governance.
///

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum SnsGovernanceDissolveState {
    DissolveDelaySeconds(u64),
    WhenDissolvedTimestampSeconds(u64),
}
