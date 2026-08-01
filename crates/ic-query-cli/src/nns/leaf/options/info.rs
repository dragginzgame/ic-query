//! Module: nns::leaf::options::info
//!
//! Responsibility: parse clap options for generic NNS leaf info commands.
//! Does not own: clap command specs, report construction, or rendering.
//! Boundary: converts info command arguments into command-runner options.

use super::NnsCommonOptions;
#[cfg(test)]
use crate::nns::NnsCommandError;
#[cfg(test)]
use crate::nns::leaf::model::NnsLeafCommandSpec;
use crate::{
    cli::{clap::required_string, common::OutputFormat},
    nns::leaf::commands::INPUT_ARG,
};
use clap::ArgMatches;
#[cfg(test)]
use std::ffi::OsString;

///
/// NnsLeafInfoOptions
///
/// Parsed options accepted by generic NNS leaf info command runners.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafInfoOptions {
    pub(in crate::nns) input: String,
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl NnsLeafInfoOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            input: required_string(matches, INPUT_ARG),
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
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
            crate::nns::leaf::commands::info_command(spec, default_source_endpoint),
            args,
        )?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}
