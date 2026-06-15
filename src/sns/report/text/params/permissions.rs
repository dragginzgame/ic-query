use super::rows::parameter_row;
use crate::sns::report::{SnsGovernanceParameters, text::common::optional_permissions_text};

pub(super) fn rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
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
