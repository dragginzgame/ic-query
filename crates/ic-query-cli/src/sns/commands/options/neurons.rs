//! Module: sns::commands::options::neurons
//!
//! Responsibility: parse SNS neuron list, refresh, and cache options.
//! Does not own: neuron cache policy, live governance reads, or reports.
//! Boundary: validates clap matches into neuron command request inputs.

#[cfg(test)]
use crate::sns::commands::{
    options::common::parse_sns_matches,
    spec::{
        sns_neuron_cache_list_command, sns_neuron_cache_status_command, sns_neuron_list_command,
        sns_neuron_refresh_command,
    },
};
use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::{OutputFormat, output_format},
    },
    sns::commands::{SnsCommandError, options::lookup::SnsLookupOptions, spec::SnsNeuronsSortArg},
};
use candid::Principal;
use clap::ArgMatches;
#[cfg(test)]
use std::ffi::OsString;

const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;

///
/// SnsNeuronsOptions
///
/// Parsed options accepted by `icq sns neuron list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) limit: u32,
    pub(in crate::sns::commands) owner_principal_id: Option<String>,
    pub(in crate::sns::commands) sort: SnsNeuronsSortArg,
    pub(in crate::sns::commands) verbose: bool,
}

///
/// SnsNeuronsCacheListOptions
///
/// Parsed options accepted by `icq sns neuron cache list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsCacheListOptions {
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
}

///
/// SnsNeuronsCacheStatusOptions
///
/// Parsed options accepted by `icq sns neuron cache status`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsCacheStatusOptions {
    pub(in crate::sns::commands) input: String,
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
}

///
/// SnsNeuronsRefreshOptions
///
/// Parsed options accepted by `icq sns neuron refresh`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsRefreshOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) page_size: u32,
    pub(in crate::sns::commands) max_pages: Option<u32>,
}

impl SnsNeuronsOptions {
    pub(in crate::sns::commands) fn from_matches(
        matches: &ArgMatches,
        network: &str,
    ) -> Result<Self, SnsCommandError> {
        let options = Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            limit: required_typed(matches, "limit"),
            owner_principal_id: typed_option::<Principal>(matches, "owner")
                .map(|principal| principal.to_text()),
            sort: required_typed(matches, "sort"),
            verbose: matches.get_flag("verbose"),
        };
        options.validate()?;
        Ok(options)
    }

    #[cfg(test)]
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_neuron_list_command(), args)?;
        Self::from_matches(&matches, ic_query::subnet_catalog::MAINNET_NETWORK)
    }

    fn validate(&self) -> Result<(), SnsCommandError> {
        if self.sort == SnsNeuronsSortArg::Api && self.limit > SNS_NEURONS_LIVE_MAX_LIMIT {
            return Err(SnsCommandError::Usage(format!(
                "`icq sns neuron list --sort api` can request at most {SNS_NEURONS_LIVE_MAX_LIMIT} live neurons at a time; refresh the cache and use `--sort <id|stake|maturity|created>` for larger limits"
            )));
        }
        if self.sort != SnsNeuronsSortArg::Api && self.owner_principal_id.is_some() {
            return Err(SnsCommandError::Usage(
                "`--owner` is supported only with `icq sns neuron list --sort api`; cached `--sort <id|stake|maturity|created>` views read the complete full-neuron cache and do not accept owner filtering".to_string(),
            ));
        }
        Ok(())
    }
}

impl SnsNeuronsCacheListOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
        }
    }

    #[cfg(test)]
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_neuron_cache_list_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}

impl SnsNeuronsCacheStatusOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: network.to_string(),
            format: output_format(matches),
        }
    }

    #[cfg(test)]
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_neuron_cache_status_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}

impl SnsNeuronsRefreshOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            page_size: required_typed(matches, "page-size"),
            max_pages: typed_option::<u32>(matches, "max-pages"),
        }
    }

    #[cfg(test)]
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_neuron_refresh_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}
