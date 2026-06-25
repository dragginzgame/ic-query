//! Module: sns::commands::options::list
//!
//! Responsibility: parse options for `icq sns list`.
//! Does not own: deployed SNS lookup, report construction, or text output.
//! Boundary: converts clap matches into the SNS list request inputs.

use crate::{
    cli::{
        clap::{required_string, required_typed},
        common::OutputFormat,
    },
    sns::commands::{
        SnsCommandError,
        options::common::parse_sns_matches,
        spec::{SnsListSortArg, sns_list_command, sns_list_usage},
    },
};
use std::ffi::OsString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsListOptions {
    pub(in crate::sns::commands) network: String,
    pub(in crate::sns::commands) format: OutputFormat,
    pub(in crate::sns::commands) source_endpoint: String,
    pub(in crate::sns::commands) verbose: bool,
    pub(in crate::sns::commands) sort: SnsListSortArg,
}

impl SnsListOptions {
    pub(in crate::sns::commands) fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_sns_matches(sns_list_command(), args, sns_list_usage)?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            verbose: matches.get_flag("verbose"),
            sort: required_typed(&matches, "sort"),
        })
    }
}
