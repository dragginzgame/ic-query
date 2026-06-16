mod args;
mod build;
mod usage;

use crate::{cli::clap::parse_matches_or_usage, nns::NnsCommandError};
use clap::{ArgMatches, Command as ClapCommand};
use std::ffi::OsString;

pub(in crate::nns) use args::{
    DRY_RUN_ARG, INPUT_ARG, LOCK_STALE_AFTER_ARG, NETWORK_ARG, OUTPUT_ARG, VERBOSE_ARG,
    network_arg, output_path_arg, refresh_lock_stale_after_arg,
};
pub(in crate::nns) use build::{command, info_command, list_command, refresh_command};
pub(in crate::nns) use usage::{info_usage, list_usage, refresh_usage, usage};

pub(in crate::nns) fn parse_leaf_matches<I>(
    command: ClapCommand,
    args: I,
    usage: impl FnOnce() -> String,
) -> Result<ArgMatches, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    parse_matches_or_usage(command, args, usage).map_err(NnsCommandError::Usage)
}
