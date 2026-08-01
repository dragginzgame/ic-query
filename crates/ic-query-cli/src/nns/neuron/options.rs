//! Parsed options for public NNS neuron commands.

use super::commands::{
    neuron_cache_status_command, neuron_info_command, neuron_list_command, neuron_refresh_command,
};
use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::output_format,
    },
    nns::{NnsCommandError, OutputFormat, leaf::NnsCommonOptions, parse_nns_matches},
};
use std::ffi::OsString;

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
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(neuron_list_command(), args)?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            limit: required_typed(&matches, "limit"),
            start_neuron_id: typed_option(&matches, "start-neuron-id"),
            verbose: matches.get_flag("verbose"),
        })
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
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(neuron_info_command(), args)?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            neuron_id: required_typed(&matches, "neuron-id"),
            verbose: matches.get_flag("verbose"),
        })
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
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(neuron_refresh_command(), args)?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            page_size: required_typed(&matches, "page-size"),
            max_pages: typed_option(&matches, "max-pages"),
        })
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
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(neuron_cache_status_command(), args)?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: output_format(&matches),
        })
    }
}
