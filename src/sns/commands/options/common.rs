//! Module: sns::commands::options::common
//!
//! Responsibility: adapt clap match parsing into SNS command usage errors.
//! Does not own: option DTOs, command specs, or command dispatch.
//! Boundary: centralizes SNS parse error mapping for option modules.

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
