use crate::{
    cli::{
        clap::{parse_matches_or_usage, required_string, required_typed},
        common::OutputFormat,
    },
    sns::commands::{
        SnsCommandError,
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
        let matches = parse_matches_or_usage(sns_list_command(), args, sns_list_usage)
            .map_err(SnsCommandError::Usage)?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            verbose: matches.get_flag("verbose"),
            sort: required_typed(&matches, "sort"),
        })
    }
}
