use super::{
    SnsCommandError,
    spec::{
        SnsListSortArg, SnsNeuronsSortArg, SnsProposalStatusArg, sns_list_command, sns_list_usage,
        sns_neurons_cache_list_command, sns_neurons_cache_list_usage,
        sns_neurons_cache_status_command, sns_neurons_cache_status_usage, sns_neurons_command,
        sns_neurons_refresh_command, sns_neurons_refresh_usage, sns_neurons_usage,
        sns_proposal_command, sns_proposal_usage, sns_proposals_command, sns_proposals_usage,
    },
};
use crate::cli::{
    clap::{parse_matches, required_string, required_typed, typed_option},
    common::OutputFormat,
};
use candid::Principal;
use clap::Command as ClapCommand;
use std::ffi::OsString;

const SNS_NEURONS_LIVE_MAX_LIMIT: u32 = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsListOptions {
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) source_endpoint: String,
    pub(super) verbose: bool,
    pub(super) sort: SnsListSortArg,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsLookupOptions {
    pub(super) input: String,
    pub(super) network: String,
    pub(super) format: OutputFormat,
    pub(super) source_endpoint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsNeuronsOptions {
    pub(super) lookup: SnsLookupOptions,
    pub(super) limit: u32,
    pub(super) owner_principal_id: Option<String>,
    pub(super) sort: SnsNeuronsSortArg,
    pub(super) verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsProposalsOptions {
    pub(super) lookup: SnsLookupOptions,
    pub(super) limit: u32,
    pub(super) before_proposal_id: Option<u64>,
    pub(super) status: SnsProposalStatusArg,
    pub(super) verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsProposalOptions {
    pub(super) lookup: SnsLookupOptions,
    pub(super) proposal_id: u64,
    pub(super) verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsNeuronsCacheListOptions {
    pub(super) network: String,
    pub(super) format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsNeuronsCacheStatusOptions {
    pub(super) input: String,
    pub(super) network: String,
    pub(super) format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SnsNeuronsRefreshOptions {
    pub(super) lookup: SnsLookupOptions,
    pub(super) page_size: u32,
    pub(super) max_pages: Option<u32>,
}

impl SnsListOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_list_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_list_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            verbose: matches.get_flag("verbose"),
            sort: required_typed(&matches, "sort"),
        })
    }
}

impl SnsLookupOptions {
    pub(super) fn parse<I>(
        args: I,
        command: fn() -> ClapCommand,
        usage: fn() -> String,
    ) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches(command(), args).map_err(|_| SnsCommandError::Usage(usage()))?;
        Ok(Self::from_matches(&matches))
    }

    fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: required_string(matches, "network"),
            format: required_typed(matches, "format"),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

impl SnsNeuronsOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_usage()))?;
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

impl SnsProposalsOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_proposals_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_proposals_usage()))?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            limit: required_typed(&matches, "limit"),
            before_proposal_id: typed_option::<u64>(&matches, "before"),
            status: required_typed(&matches, "status"),
            verbose: matches.get_flag("verbose"),
        })
    }
}

impl SnsProposalOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_proposal_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_proposal_usage()))?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            proposal_id: required_typed(&matches, "proposal-id"),
            verbose: matches.get_flag("verbose"),
        })
    }
}

impl SnsNeuronsCacheListOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_cache_list_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_cache_list_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsNeuronsCacheStatusOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_cache_status_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_cache_status_usage()))?;
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
        })
    }
}

impl SnsNeuronsRefreshOptions {
    pub(super) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_neurons_refresh_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_neurons_refresh_usage()))?;
        Ok(Self {
            lookup: SnsLookupOptions::from_matches(&matches),
            page_size: required_typed(&matches, "page-size"),
            max_pages: typed_option::<u32>(&matches, "max-pages"),
        })
    }
}
