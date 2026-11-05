//! Module: sns::commands::spec::commands::neurons::sort
//!
//! Responsibility: build the clap value argument for SNS neuron row sorting.
//! Does not own: sort implementation, cache selection, or report views.
//! Boundary: keeps the CLI sort vocabulary aligned with SNS neuron reports.

use crate::cli::clap::{choice_parser, value_arg};
use ic_query::sns::SnsNeuronsCacheSort;

pub(super) fn neurons_sort_arg() -> clap::Arg {
    value_arg("sort")
        .long("sort")
        .value_name("api|id|stake|maturity|created")
        .default_value("api")
        .value_parser(choice_parser(
            &[
                None,
                Some(SnsNeuronsCacheSort::Id),
                Some(SnsNeuronsCacheSort::Stake),
                Some(SnsNeuronsCacheSort::Maturity),
                Some(SnsNeuronsCacheSort::Created),
            ],
            |sort| sort.map_or("api", SnsNeuronsCacheSort::as_str),
        ))
        .help("Row order; api uses a bounded live query, other sorts read the complete cache")
}
