use crate::{
    cli::{
        clap::{parse_matches, required_string, required_typed},
        common::OutputFormat,
    },
    sns::commands::SnsCommandError,
};
use clap::Command as ClapCommand;
use std::ffi::OsString;

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
        usage: fn() -> String,
    ) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_matches(command(), args).map_err(|_| SnsCommandError::Usage(usage()))?;
        Ok(Self::from_matches(&matches))
    }

    pub(super) fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            input: required_string(matches, "input"),
            network: required_string(matches, "network"),
            format: required_typed(matches, "format"),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}
