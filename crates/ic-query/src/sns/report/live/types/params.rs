//! Module: sns::report::live::types::params
//!
//! Responsibility: complete SNS Governance nervous-system parameter wire types.
//! Does not own: parameter projection, validation, or text rendering.
//! Boundary: mirrors the full native response, including default followees.

use super::SnsGovernanceFollowees;
use crate::sns::report::{
    SnsCustomProposalCriticality, SnsNeuronPermissionList, SnsVotingRewardsParameters,
};
use candid::{CandidType, Deserialize};

///
/// SnsDefaultFolloweesWire
///
/// Native function-keyed default-followee map.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsDefaultFolloweesWire {
    pub(in crate::sns::report::live) followees: Vec<(u64, SnsGovernanceFollowees)>,
}

///
/// SnsGovernanceParametersWire
///
/// Complete native nervous-system parameter response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceParametersWire {
    pub(in crate::sns::report::live) default_followees: Option<SnsDefaultFolloweesWire>,
    pub(in crate::sns::report::live) max_dissolve_delay_seconds: Option<u64>,
    pub(in crate::sns::report::live) max_dissolve_delay_bonus_percentage: Option<u64>,
    pub(in crate::sns::report::live) max_followees_per_function: Option<u64>,
    pub(in crate::sns::report::live) neuron_claimer_permissions: Option<SnsNeuronPermissionList>,
    pub(in crate::sns::report::live) neuron_minimum_stake_e8s: Option<u64>,
    pub(in crate::sns::report::live) max_neuron_age_for_age_bonus: Option<u64>,
    pub(in crate::sns::report::live) initial_voting_period_seconds: Option<u64>,
    pub(in crate::sns::report::live) neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
    pub(in crate::sns::report::live) reject_cost_e8s: Option<u64>,
    pub(in crate::sns::report::live) max_proposals_to_keep_per_action: Option<u32>,
    pub(in crate::sns::report::live) wait_for_quiet_deadline_increase_seconds: Option<u64>,
    pub(in crate::sns::report::live) max_number_of_neurons: Option<u64>,
    pub(in crate::sns::report::live) transaction_fee_e8s: Option<u64>,
    pub(in crate::sns::report::live) max_number_of_proposals_with_ballots: Option<u64>,
    pub(in crate::sns::report::live) max_age_bonus_percentage: Option<u64>,
    pub(in crate::sns::report::live) neuron_grantable_permissions: Option<SnsNeuronPermissionList>,
    pub(in crate::sns::report::live) voting_rewards_parameters: Option<SnsVotingRewardsParameters>,
    pub(in crate::sns::report::live) maturity_modulation_disabled: Option<bool>,
    pub(in crate::sns::report::live) max_number_of_principals_per_neuron: Option<u64>,
    pub(in crate::sns::report::live) automatically_advance_target_version: Option<bool>,
    pub(in crate::sns::report::live) custom_proposal_criticality:
        Option<SnsCustomProposalCriticality>,
}
