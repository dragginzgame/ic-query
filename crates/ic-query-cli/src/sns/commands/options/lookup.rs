//! Module: sns::commands::options::lookup
//!
//! Responsibility: parse shared SNS lookup command options.
//! Does not own: command-specific option fields or report requests.
//! Boundary: captures network, format, endpoint, and SNS selector inputs.

use crate::{
    cli::{
        clap::required_string,
        common::{OutputFormat, output_format},
    },
    sns::commands::{SnsCommandError, options::common::parse_sns_matches},
};
use clap::Command as ClapCommand;
use std::ffi::OsString;

///
/// SnsLookupOptions
///
/// Common selector and source options shared by SNS lookup commands.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsLookupOptions {
    pub(in crate::sns::commands) input: String,
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
    pub(in crate::sns::commands) source_endpoint: String,
}

impl SnsLookupOptions {
    pub(in crate::sns::commands) fn parse<I>(
        args: I,
        command: fn() -> ClapCommand,
    ) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(command(), args)?;
        Ok(Self::from_matches(&matches))
    }

    pub(super) fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: required_string(matches, "network"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}
