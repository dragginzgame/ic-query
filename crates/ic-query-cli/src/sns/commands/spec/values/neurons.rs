//! Module: sns::commands::spec::values::neurons
//!
//! Responsibility: clap value enum for SNS neuron list sorting.
//! Does not own: neuron cache selection or report sorting behavior.
//! Boundary: converts CLI sort values into report-model sort values.

use crate::cli::value_enum_with_conversion;
use ic_query::sns::SnsNeuronsSort;

value_enum_with_conversion! {
    ///
    /// SnsNeuronsSortArg
    ///
    /// Command-local clap value accepted by `icq sns neuron list <SNS> --sort`.
    ///

    pub(in crate::sns::commands) enum SnsNeuronsSortArg => SnsNeuronsSort {
        #[default]
        Api,
        Id,
        Stake,
        Maturity,
        Created,
    }
}
