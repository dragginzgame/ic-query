//! Module: sns::commands::spec::commands::neurons::sort
//!
//! Responsibility: build the clap value argument for SNS neuron row sorting.
//! Does not own: sort implementation, cache selection, or report views.
//! Boundary: keeps the CLI sort vocabulary aligned with SNS neuron reports.

use crate::{cli::clap::value_arg, sns::commands::spec::values::SnsNeuronsSortArg};

pub(super) fn neurons_sort_arg() -> clap::Arg {
    value_arg("sort")
        .long("sort")
        .value_name("api|id|stake|maturity|created")
        .default_value("api")
        .value_parser(clap::value_parser!(SnsNeuronsSortArg))
        .help("Row order; api uses a bounded live query, other sorts read the complete cache")
}
