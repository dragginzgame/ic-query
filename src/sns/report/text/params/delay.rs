use super::rows::parameter_row;
use crate::sns::report::{
    SnsGovernanceParameters,
    text::common::{optional_duration_text, optional_percentage_text},
};

pub(super) fn rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
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
