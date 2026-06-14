use super::{
    commands::{
        DRY_RUN_ARG, INPUT_ARG, LOCK_STALE_AFTER_ARG, NETWORK_ARG, OUTPUT_ARG, VERBOSE_ARG,
        info_command, info_usage, list_command, list_usage, parse_leaf_matches, refresh_command,
        refresh_usage,
    },
    model::NnsLeafCommandSpec,
};
use crate::{
    cli::{
        clap::{required_string, required_typed, typed_option},
        common::{FORMAT_ARG, OutputFormat, SOURCE_ENDPOINT_ARG},
    },
    nns::NnsCommandError,
};
use clap::ArgMatches;
use std::{ffi::OsString, path::PathBuf};

///
/// NnsCommonOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsCommonOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl NnsCommonOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            network: required_string(matches, NETWORK_ARG),
            format: required_typed(matches, FORMAT_ARG),
            source_endpoint: required_string(matches, SOURCE_ENDPOINT_ARG),
        }
    }
}

///
/// NnsLeafListOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) verbose: bool,
}

impl NnsLeafListOptions {
    pub(in crate::nns) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_leaf_matches(list_command(spec, default_source_endpoint), args, || {
                list_usage(spec, default_source_endpoint)
            })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            verbose: matches.get_flag(VERBOSE_ARG),
        })
    }
}

///
/// NnsLeafInfoOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafInfoOptions {
    pub(in crate::nns) input: String,
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl NnsLeafInfoOptions {
    pub(in crate::nns) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_leaf_matches(info_command(spec, default_source_endpoint), args, || {
                info_usage(spec, default_source_endpoint)
            })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            input: required_string(&matches, INPUT_ARG),
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}

///
/// NnsLeafRefreshOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) lock_stale_after_seconds: u64,
    pub(in crate::nns) dry_run: bool,
    pub(in crate::nns) output_path: Option<PathBuf>,
}

impl NnsLeafRefreshOptions {
    pub(in crate::nns) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_leaf_matches(refresh_command(spec, default_source_endpoint), args, || {
                refresh_usage(spec, default_source_endpoint)
            })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(&matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
            output_path: typed_option(&matches, OUTPUT_ARG),
        })
    }
}
