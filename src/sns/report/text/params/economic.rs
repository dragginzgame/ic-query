use super::rows::parameter_row;
use crate::sns::report::{SnsGovernanceParameters, text::common::optional_e8s_text};

pub(super) fn rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
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
