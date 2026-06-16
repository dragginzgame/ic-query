use super::{common::parse_sns_matches, lookup::SnsLookupOptions};
use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::OutputFormat,
    },
    sns::commands::{
        SnsCommandError,
        spec::{
            SnsNeuronsSortArg, sns_neurons_cache_list_command, sns_neurons_cache_list_usage,
            sns_neurons_cache_status_command, sns_neurons_cache_status_usage, sns_neurons_command,
            sns_neurons_refresh_command, sns_neurons_refresh_usage, sns_neurons_usage,
        },
    },
};
use candid::Principal;
use std::ffi::OsString;

const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) limit: u32,
    pub(in crate::sns::commands) owner_principal_id: Option<String>,
    pub(in crate::sns::commands) sort: SnsNeuronsSortArg,
    pub(in crate::sns::commands) verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsCacheListOptions {
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsCacheStatusOptions {
    pub(in crate::sns::commands) input: String,
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsNeuronsRefreshOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) page_size: u32,
    pub(in crate::sns::commands) max_pages: Option<u32>,
}

impl SnsNeuronsOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_neurons_command(), args, sns_neurons_usage)?;
        let options = Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            limit: required_typed(&matches, "limit"),
            owner_principal_id: typed_option::<Principal>(&matches, "owner")
                .map(|principal| principal.to_text()),
            sort: required_typed(&matches, "sort"),
            verbose: matches.get_flag("verbose"),
        };
        options.validate()?;
        Ok(options)
    }

    fn validate(&self) -> Result<(), SnsCommandError> {
        if self.sort == SnsNeuronsSortArg::Api && self.limit > SNS_NEURONS_LIVE_MAX_LIMIT {
            return Err(SnsCommandError::Usage(format!(
                "`icq sns neurons --sort api` can request at most {SNS_NEURONS_LIVE_MAX_LIMIT} live neurons at a time; refresh the cache and use `--sort <id|stake|maturity|created>` for larger limits"
            )));
        }
        Ok(())
    }
}

impl SnsNeuronsCacheListOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(
            sns_neurons_cache_list_command(),
            args,
            sns_neurons_cache_list_usage,
        )?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsNeuronsCacheStatusOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(
            sns_neurons_cache_status_command(),
            args,
            sns_neurons_cache_status_usage,
        )?;
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsNeuronsRefreshOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(
            sns_neurons_refresh_command(),
            args,
            sns_neurons_refresh_usage,
        )?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            page_size: required_typed(&matches, "page-size"),
            max_pages: typed_option::<u32>(&matches, "max-pages"),
        })
    }
}
