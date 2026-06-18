//! Module: sns::report::text::params::rows
//!
//! Responsibility: construct grouped SNS parameter table rows.
//! Does not own: parameter fetching, report construction, or table rendering.
//! Boundary: keeps parameter row grouping together without one-function modules.

use crate::sns::report::{
    SnsGovernanceParameters,
    text::common::{
        comma_join_u64, optional_basis_points_text, optional_bool_text, optional_duration_text,
        optional_e8s_text, optional_percentage_text, optional_permissions_text, optional_u32_text,
        optional_u64_text,
    },
};

pub(in crate::sns::report::text::params) fn sns_params_text_rows(
    parameters: &SnsGovernanceParameters,
) -> Vec<[String; 2]> {
    [
        economic_rows(parameters),
        delay_rows(parameters),
        limits_rows(parameters),
        permissions_rows(parameters),
        reward_rows(parameters),
    ]
    .concat()
}

fn economic_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "neuron_minimum_stake",
            optional_e8s_text(parameters.neuron_minimum_stake_e8s),
        ),
        parameter_row(
            "transaction_fee",
            optional_e8s_text(parameters.transaction_fee_e8s),
        ),
        parameter_row("reject_cost", optional_e8s_text(parameters.reject_cost_e8s)),
    ]
}

fn delay_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "max_dissolve_delay",
            optional_duration_text(parameters.max_dissolve_delay_seconds),
        ),
        parameter_row(
            "max_dissolve_delay_bonus",
            optional_percentage_text(parameters.max_dissolve_delay_bonus_percentage),
        ),
        parameter_row(
            "max_neuron_age_for_age_bonus",
            optional_duration_text(parameters.max_neuron_age_for_age_bonus),
        ),
        parameter_row(
            "max_age_bonus",
            optional_percentage_text(parameters.max_age_bonus_percentage),
        ),
        parameter_row(
            "initial_voting_period",
            optional_duration_text(parameters.initial_voting_period_seconds),
        ),
        parameter_row(
            "wait_for_quiet_deadline_increase",
            optional_duration_text(parameters.wait_for_quiet_deadline_increase_seconds),
        ),
        parameter_row(
            "minimum_dissolve_delay_to_vote",
            optional_duration_text(parameters.neuron_minimum_dissolve_delay_to_vote_seconds),
        ),
    ]
}

fn limits_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "max_followees_per_function",
            optional_u64_text(parameters.max_followees_per_function),
        ),
        parameter_row(
            "max_proposals_to_keep_per_action",
            optional_u32_text(parameters.max_proposals_to_keep_per_action),
        ),
        parameter_row(
            "max_number_of_neurons",
            optional_u64_text(parameters.max_number_of_neurons),
        ),
        parameter_row(
            "max_number_of_proposals_with_ballots",
            optional_u64_text(parameters.max_number_of_proposals_with_ballots),
        ),
        parameter_row(
            "max_number_of_principals_per_neuron",
            optional_u64_text(parameters.max_number_of_principals_per_neuron),
        ),
        parameter_row(
            "maturity_modulation_disabled",
            optional_bool_text(parameters.maturity_modulation_disabled),
        ),
        parameter_row(
            "automatically_advance_target_version",
            optional_bool_text(parameters.automatically_advance_target_version),
        ),
    ]
}

fn permissions_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "neuron_claimer_permissions",
            optional_permissions_text(parameters.neuron_claimer_permissions.as_ref()),
        ),
        parameter_row(
            "neuron_grantable_permissions",
            optional_permissions_text(parameters.neuron_grantable_permissions.as_ref()),
        ),
    ]
}

fn reward_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    let rewards = parameters.voting_rewards_parameters.as_ref();
    vec![
        parameter_row(
            "voting_reward_initial_rate",
            optional_basis_points_text(
                rewards.and_then(|rewards| rewards.initial_reward_rate_basis_points),
            ),
        ),
        parameter_row(
            "voting_reward_final_rate",
            optional_basis_points_text(
                rewards.and_then(|rewards| rewards.final_reward_rate_basis_points),
            ),
        ),
        parameter_row(
            "voting_reward_transition_duration",
            optional_duration_text(
                rewards.and_then(|rewards| rewards.reward_rate_transition_duration_seconds),
            ),
        ),
        parameter_row(
            "voting_reward_round_duration",
            optional_duration_text(rewards.and_then(|rewards| rewards.round_duration_seconds)),
        ),
        parameter_row(
            "additional_critical_native_actions",
            parameters.custom_proposal_criticality.as_ref().map_or_else(
                || "-".to_string(),
                |criticality| comma_join_u64(&criticality.additional_critical_native_action_ids),
            ),
        ),
    ]
}

fn parameter_row(name: &str, value: String) -> [String; 2] {
    [name.to_string(), value]
}
