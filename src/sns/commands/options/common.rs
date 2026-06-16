use crate::{cli::clap::parse_matches_or_usage, sns::commands::SnsCommandError};
use clap::{ArgMatches, Command as ClapCommand};
use std::ffi::OsString;

pub(super) fn parse_sns_matches<I>(
    command: ClapCommand,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<ArgMatches, SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    parse_matches_or_usage(command, args, usage).map_err(SnsCommandError::Usage)
}
