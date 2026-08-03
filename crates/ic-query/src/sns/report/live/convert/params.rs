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
mod tests {
    use super::*;
    use crate::sns::report::{SnsNeuronId, live::types::SnsGovernanceFollowees};

    #[test]
    fn complete_parameter_conversion_preserves_default_followees() {
        let mut parameters = empty_parameters();
        parameters.default_followees = Some(SnsDefaultFolloweesWire {
            followees: vec![
                (
                    7,
                    SnsGovernanceFollowees {
                        followees: vec![SnsNeuronId { id: vec![0xab; 32] }],
                    },
                ),
                (
                    9,
                    SnsGovernanceFollowees {
                        followees: vec![
                            SnsNeuronId { id: vec![0xcd; 32] },
                            SnsNeuronId { id: vec![0xef; 32] },
                        ],
                    },
                ),
            ],
        });

        let converted = sns_governance_parameters(parameters);
        let defaults = converted.default_followees.expect("default followees");

        assert_eq!(defaults.followees[0].function_id, 7);
        assert_eq!(defaults.followees[0].followee_neuron_ids, ["ab".repeat(32)]);
        assert_eq!(defaults.followees[1].function_id, 9);
        assert_eq!(
            defaults.followees[1].followee_neuron_ids,
            ["cd".repeat(32), "ef".repeat(32)]
        );
    }

    const fn empty_parameters() -> SnsGovernanceParametersWire {
        SnsGovernanceParametersWire {
            default_followees: None,
            max_dissolve_delay_seconds: None,
            max_dissolve_delay_bonus_percentage: None,
            max_followees_per_function: None,
            neuron_claimer_permissions: None,
            neuron_minimum_stake_e8s: None,
            max_neuron_age_for_age_bonus: None,
            initial_voting_period_seconds: None,
            neuron_minimum_dissolve_delay_to_vote_seconds: None,
            reject_cost_e8s: None,
            max_proposals_to_keep_per_action: None,
            wait_for_quiet_deadline_increase_seconds: None,
            max_number_of_neurons: None,
            transaction_fee_e8s: None,
            max_number_of_proposals_with_ballots: None,
            max_age_bonus_percentage: None,
            neuron_grantable_permissions: None,
            voting_rewards_parameters: None,
            maturity_modulation_disabled: None,
            max_number_of_principals_per_neuron: None,
            automatically_advance_target_version: None,
            custom_proposal_criticality: None,
        }
    }
}
