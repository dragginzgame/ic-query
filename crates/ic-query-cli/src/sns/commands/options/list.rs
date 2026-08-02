//! Module: sns::commands::options::list
//!
//! Responsibility: parse options for `icq sns list`.
//! Does not own: deployed SNS lookup, report construction, or text output.
//! Boundary: converts clap matches into the SNS list request inputs.

use crate::{
    cli::{
        clap::{required_string, required_typed},
        common::{OutputFormat, output_format},
    },
    sns::commands::spec::SnsListSortArg,
};
use clap::ArgMatches;

///
/// SnsListOptions
///
/// Parsed options accepted by `icq sns list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsListOptions {
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
    pub(in crate::sns::commands) source_endpoint: String,
    pub(in crate::sns::commands) verbose: bool,
    pub(in crate::sns::commands) sort: SnsListSortArg,
}

impl SnsListOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
            verbose: matches.get_flag("verbose"),
            sort: required_typed(matches, "sort"),
        }
    }
}
