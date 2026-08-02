//! Parsed options for public NNS neuron commands.

use crate::{
    cli::{
        clap::{required_typed, typed_option},
        common::output_format,
    },
    nns::{OutputFormat, leaf::NnsCommonOptions},
};
use clap::ArgMatches;

///
/// NnsNeuronListOptions
///
/// Parsed options for one public NNS neuron list view.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNeuronListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) limit: u32,
    pub(in crate::nns) start_neuron_id: Option<u64>,
    pub(in crate::nns) verbose: bool,
}

impl NnsNeuronListOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
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
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) neuron_id: u64,
    pub(in crate::nns) verbose: bool,
}

impl NnsNeuronInfoOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            neuron_id: required_typed(matches, "neuron-id"),
            verbose: matches.get_flag("verbose"),
        }
    }
}

///
/// NnsNeuronRefreshOptions
///
/// Parsed options for one complete public NNS neuron refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNeuronRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) page_size: u32,
    pub(in crate::nns) max_pages: Option<u32>,
}

impl NnsNeuronRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            page_size: required_typed(matches, "page-size"),
            max_pages: typed_option(matches, "max-pages"),
        }
    }
}

///
/// NnsNeuronCacheOptions
///
/// Parsed options for local public NNS neuron cache inspection.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsNeuronCacheOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
}

impl NnsNeuronCacheOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
        }
    }
}
