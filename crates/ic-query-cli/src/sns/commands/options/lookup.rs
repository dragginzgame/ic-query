//! Module: sns::commands::options::lookup
//!
//! Responsibility: parse shared SNS lookup command options.
//! Does not own: command-specific option fields or report requests.
//! Boundary: captures network, format, endpoint, and SNS selector inputs.

use crate::cli::{
    clap::required_string,
    common::{OutputFormat, output_format},
};
use clap::ArgMatches;

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
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}
