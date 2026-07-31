//! Module: ic
//!
//! Responsibility: parse and dispatch official IC Dashboard command families.
//! Does not own: REST transport, report construction, or text rendering.
//! Boundary: exposes live-only canister command wiring to the top-level CLI.

use crate::{
    cli::{
        clap::{
            parse_matches_or_usage, parse_required_subcommand_or_usage, passthrough_subcommand,
            render_help, required_string, required_typed, value_arg,
        },
        common::{
            COLLECTION_MODE_LIVE, CurrentUnixSecsError, OutputFormat, collection_help,
            current_unix_secs, format_arg, source_endpoint_arg, write_text_or_json,
        },
        help::collect_args_or_print_help_or_version,
    },
    version_text,
};
use clap::Command as ClapCommand;
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcCanisterRequest, IcHostError, build_ic_canister_report,
    ic_canister_report_text,
};
use std::{ffi::OsString, io};
use thiserror::Error as ThisError;

const CANISTER_INFO_HELP_AFTER: &str = "\
Examples:
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --source-endpoint https://ic-api.internetcomputer.org/api/v3

The official Dashboard API is an off-chain analytics authority. Its response
is not presented as certified IC state or an exact point-in-time snapshot.";

///
/// IcCommandError
///
/// Errors surfaced while parsing or running an `icq ic` command.
///

#[derive(Debug, ThisError)]
pub enum IcCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Host(#[from] IcHostError),

    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn run<I>(args: I) -> Result<(), IcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(ic_command(), args, usage)
        .map_err(IcCommandError::Usage)?;
    match command.as_str() {
        "canister" => run_canister(args),
        _ => unreachable!("ic dispatch command only defines known commands"),
    }
}

fn run_canister<I>(args: I) -> Result<(), IcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, canister_usage) else {
        return Ok(());
    };
    let (command, args) =
        parse_required_subcommand_or_usage(canister_command(), args, canister_usage)
            .map_err(IcCommandError::Usage)?;
    match command.as_str() {
        "info" => run_canister_info(args),
        _ => unreachable!("ic canister dispatch command only defines known commands"),
    }
}

fn run_canister_info<I>(args: I) -> Result<(), IcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, canister_info_usage) else {
        return Ok(());
    };
    let options = CanisterInfoOptions::parse(args)?;
    let request = IcCanisterRequest::new(
        options.source_endpoint,
        current_unix_secs()?,
        options.canister_id,
    );
    let report = build_ic_canister_report(&request)?;
    write_text_or_json(options.format, &report, ic_canister_report_text)
}

fn command_args<I>(args: I, usage: impl FnOnce() -> String) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_help_or_version(args, usage, version_text())
}

fn ic_command() -> ClapCommand {
    ClapCommand::new("ic")
        .bin_name("icq ic")
        .about("Inspect official IC Dashboard metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("canister").about("Inspect deployed canister metadata"),
        ))
}

fn canister_command() -> ClapCommand {
    ClapCommand::new("canister")
        .bin_name("icq ic canister")
        .about("Inspect deployed canister metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about("Show one canister from the official Dashboard API"),
        ))
}

fn canister_info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq ic canister info")
        .about("Show one canister from the official Dashboard API")
        .disable_help_flag(true)
        .arg(
            value_arg("canister-id")
                .required(true)
                .value_name("canister-id")
                .help("Canister principal"),
        )
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API base endpoint"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            CANISTER_INFO_HELP_AFTER,
        ))
}

fn usage() -> String {
    render_help(ic_command())
}

fn canister_usage() -> String {
    render_help(canister_command())
}

fn canister_info_usage() -> String {
    render_help(canister_info_command())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanisterInfoOptions {
    canister_id: String,
    format: OutputFormat,
    source_endpoint: String,
}

impl CanisterInfoOptions {
    fn parse<I>(args: I) -> Result<Self, IcCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(canister_info_command(), args, canister_info_usage)
            .map_err(IcCommandError::Usage)?;
        Ok(Self {
            canister_id: required_string(&matches, "canister-id"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
        })
    }
}

#[cfg(test)]
mod tests;
