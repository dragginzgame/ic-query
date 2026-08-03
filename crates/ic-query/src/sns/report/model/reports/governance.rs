//! Module: sns::report::model::reports::governance
//!
//! Responsibility: SNS governance parameter DTOs shared by reports.
//! Does not own: live governance fetches, parameter rendering, or defaults.
//! Boundary: preserves the complete native parameter response in JSON-friendly report fields.

use candid::{CandidType, Deserialize};
use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// SnsGovernanceParameters
///
/// Serializable complete SNS governance parameter set.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsGovernanceParameters {
    /// Native default followee map when Governance supplied one.
    pub default_followees: Option<SnsDefaultFollowees>,
    pub max_dissolve_delay_seconds: Option<u64>,
    pub max_dissolve_delay_bonus_percentage: Option<u64>,
    pub max_followees_per_function: Option<u64>,
    pub neuron_claimer_permissions: Option<SnsNeuronPermissionList>,
    pub neuron_minimum_stake_e8s: Option<u64>,
    pub max_neuron_age_for_age_bonus: Option<u64>,
    pub initial_voting_period_seconds: Option<u64>,
    pub neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
    pub reject_cost_e8s: Option<u64>,
    pub max_proposals_to_keep_per_action: Option<u32>,
    pub wait_for_quiet_deadline_increase_seconds: Option<u64>,
    pub max_number_of_neurons: Option<u64>,
    pub transaction_fee_e8s: Option<u64>,
    pub max_number_of_proposals_with_ballots: Option<u64>,
    pub max_age_bonus_percentage: Option<u64>,
    pub neuron_grantable_permissions: Option<SnsNeuronPermissionList>,
    pub voting_rewards_parameters: Option<SnsVotingRewardsParameters>,
    pub maturity_modulation_disabled: Option<bool>,
    pub max_number_of_principals_per_neuron: Option<u64>,
    pub automatically_advance_target_version: Option<bool>,
    pub custom_proposal_criticality: Option<SnsCustomProposalCriticality>,
}

///
/// SnsNeuronPermissionList
///
/// Serializable and Candid-compatible list of SNS neuron permission codes.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsNeuronPermissionList {
    pub permissions: Vec<i32>,
}

///
/// SnsDefaultFollowees
///
/// Complete native default-followee map projected into stable report rows.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsDefaultFollowees {
    /// Function-keyed default followee entries in native response order.
    pub followees: Vec<SnsDefaultFolloweesRow>,
}

///
/// SnsDefaultFolloweesRow
///
/// Default neuron followees for one nervous-system function identifier.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsDefaultFolloweesRow {
    /// Native nervous-system function identifier.
    pub function_id: u64,
    /// Full followee neuron identifiers as lowercase hexadecimal text.
    pub followee_neuron_ids: Vec<String>,
}

///
/// SnsVotingRewardsParameters
///
/// Serializable and Candid-compatible SNS voting reward parameter set.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsVotingRewardsParameters {
    pub final_reward_rate_basis_points: Option<u64>,
    pub initial_reward_rate_basis_points: Option<u64>,
    pub reward_rate_transition_duration_seconds: Option<u64>,
    pub round_duration_seconds: Option<u64>,
}

///
/// SnsCustomProposalCriticality
///
/// Serializable and Candid-compatible SNS custom proposal criticality config.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnsCustomProposalCriticality {
    pub additional_critical_native_action_ids: Vec<u64>,
}
