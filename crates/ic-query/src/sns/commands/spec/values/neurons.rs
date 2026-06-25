//! Module: sns::commands::spec::values::neurons
//!
//! Responsibility: clap value enum for SNS neuron list sorting.
//! Does not own: neuron cache selection or report sorting behavior.
//! Boundary: converts CLI sort values into report-model sort values.

use crate::sns::report::SnsNeuronsSort;
use clap::ValueEnum;

///
/// SnsNeuronsSortArg
///
/// Command-local clap value accepted by `icq sns neurons --sort`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::sns::commands) enum SnsNeuronsSortArg {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}

impl From<SnsNeuronsSortArg> for SnsNeuronsSort {
    fn from(value: SnsNeuronsSortArg) -> Self {
        match value {
            SnsNeuronsSortArg::Api => Self::Api,
            SnsNeuronsSortArg::Id => Self::Id,
            SnsNeuronsSortArg::Stake => Self::Stake,
            SnsNeuronsSortArg::Maturity => Self::Maturity,
            SnsNeuronsSortArg::Created => Self::Created,
        }
    }
}
