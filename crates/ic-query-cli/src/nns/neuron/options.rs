//! Parsed options for public NNS neuron commands.

use crate::cli::clap::required_typed;
use crate::{cli::clap::typed_option, nns::leaf::NnsCommonOptions};
use clap::ArgMatches;

///
/// NnsNeuronListOptions
///
/// Parsed options for one public NNS neuron list view.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNeuronListOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) limit: u32,
    pub(in crate::nns) start_neuron_id: Option<u64>,
    pub(in crate::nns) verbose: bool,
}

impl NnsNeuronListOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            common,
            limit: required_typed(matches, "limit"),
            start_neuron_id: typed_option(matches, "start-neuron-id"),
            verbose: matches.get_flag("verbose"),
        }
    }
}

///
/// NnsNeuronInfoOptions
///
/// Parsed options for one public NNS neuron detail view.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNeuronInfoOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) neuron_id: u64,
    pub(in crate::nns) verbose: bool,
}

impl NnsNeuronInfoOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            common,
            neuron_id: required_typed(matches, "neuron-id"),
            verbose: matches.get_flag("verbose"),
        }
    }
}
