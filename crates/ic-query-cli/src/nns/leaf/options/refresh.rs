//! Module: nns::leaf::options::refresh
//!
//! Responsibility: parse clap options for generic NNS leaf refresh commands.
//! Does not own: clap command specs, cache writes, or report construction.
//! Boundary: converts refresh command arguments into command-runner options.

use super::NnsCommonOptions;
#[cfg(test)]
use crate::nns::NnsCommandError;
#[cfg(test)]
use crate::nns::leaf::model::NnsLeafCommandSpec;
use crate::{
    cli::{
        clap::{required_typed, typed_option},
        common::OutputFormat,
    },
    nns::leaf::commands::{DRY_RUN_ARG, LOCK_STALE_AFTER_ARG, OUTPUT_ARG},
};
use clap::ArgMatches;
#[cfg(test)]
use std::ffi::OsString;
use std::path::PathBuf;

///
/// NnsLeafRefreshOptions
///
/// Parsed options accepted by generic NNS leaf refresh command runners.
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
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
            output_path: typed_option(matches, OUTPUT_ARG),
        }
    }

    #[cfg(test)]
    pub(in crate::nns) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::nns::parse_nns_matches(
            crate::nns::leaf::commands::refresh_command(spec, default_source_endpoint),
            args,
        )?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}
