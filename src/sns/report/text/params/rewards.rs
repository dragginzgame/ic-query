use super::rows::parameter_row;
use crate::sns::report::{
    SnsGovernanceParameters,
    text::common::{comma_join_u64, optional_basis_points_text, optional_duration_text},
};

pub(super) fn rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
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
