//! Module: sns::report::live::convert::params
//!
//! Responsibility: convert complete native SNS parameter responses into report DTOs.
//! Does not own: Governance queries, parameter validation, or text rendering.
//! Boundary: converts default-followee neuron ids to canonical lowercase hexadecimal text.

use crate::{
    hex::hex_bytes,
    sns::report::{
        SnsDefaultFollowees, SnsDefaultFolloweesRow, SnsGovernanceParameters,
        live::types::{SnsDefaultFolloweesWire, SnsGovernanceParametersWire},
    },
};

pub(in crate::sns::report::live) fn sns_governance_parameters(
    parameters: SnsGovernanceParametersWire,
) -> SnsGovernanceParameters {
    SnsGovernanceParameters {
        default_followees: parameters.default_followees.map(default_followees),
        max_dissolve_delay_seconds: parameters.max_dissolve_delay_seconds,
        max_dissolve_delay_bonus_percentage: parameters.max_dissolve_delay_bonus_percentage,
        max_followees_per_function: parameters.max_followees_per_function,
        neuron_claimer_permissions: parameters.neuron_claimer_permissions,
        neuron_minimum_stake_e8s: parameters.neuron_minimum_stake_e8s,
        max_neuron_age_for_age_bonus: parameters.max_neuron_age_for_age_bonus,
        initial_voting_period_seconds: parameters.initial_voting_period_seconds,
        neuron_minimum_dissolve_delay_to_vote_seconds: parameters
            .neuron_minimum_dissolve_delay_to_vote_seconds,
        reject_cost_e8s: parameters.reject_cost_e8s,
        max_proposals_to_keep_per_action: parameters.max_proposals_to_keep_per_action,
        wait_for_quiet_deadline_increase_seconds: parameters
            .wait_for_quiet_deadline_increase_seconds,
        max_number_of_neurons: parameters.max_number_of_neurons,
        transaction_fee_e8s: parameters.transaction_fee_e8s,
        max_number_of_proposals_with_ballots: parameters.max_number_of_proposals_with_ballots,
        max_age_bonus_percentage: parameters.max_age_bonus_percentage,
        neuron_grantable_permissions: parameters.neuron_grantable_permissions,
        voting_rewards_parameters: parameters.voting_rewards_parameters,
        maturity_modulation_disabled: parameters.maturity_modulation_disabled,
        max_number_of_principals_per_neuron: parameters.max_number_of_principals_per_neuron,
        automatically_advance_target_version: parameters.automatically_advance_target_version,
        custom_proposal_criticality: parameters.custom_proposal_criticality,
    }
}

fn default_followees(defaults: SnsDefaultFolloweesWire) -> SnsDefaultFollowees {
    SnsDefaultFollowees {
        followees: defaults
            .followees
            .into_iter()
            .map(|(function_id, followees)| SnsDefaultFolloweesRow {
                function_id,
                followee_neuron_ids: followees
                    .followees
                    .into_iter()
                    .map(|neuron_id| hex_bytes(&neuron_id.id))
                    .collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests;
