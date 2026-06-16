use super::rows::parameter_row;
use crate::sns::report::{
    SnsGovernanceParameters,
    text::common::{optional_bool_text, optional_u32_text, optional_u64_text},
};

pub(super) fn rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
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
