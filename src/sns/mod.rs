pub mod report;

use crate::{
    cli::{
        clap::{
            flag_arg, parse_matches, parse_required_subcommand, passthrough_subcommand,
            render_help, required_string, required_typed, value_arg,
        },
        common::{
            OutputFormat, current_unix_secs, format_arg, source_endpoint_arg, write_text_or_json,
        },
        globals::internal_network_arg,
        help::print_help_or_version,
    },
    sns::report::{
        DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsInfoRequest, SnsListRequest, SnsListSort,
        build_sns_info_report, build_sns_list_report, sns_info_report_text, sns_list_report_text,
    },
    version_text,
};
use clap::{Command as ClapCommand, ValueEnum};
use std::{ffi::OsString, io};
use thiserror::Error as ThisError;

const SNS_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns list
  icq sns list --sort name
  icq sns list --verbose
  icq --network ic sns list --format json
  icq sns list --source-endpoint https://icp-api.io";

const SNS_INFO_HELP_AFTER: &str = "\
Examples:
  icq sns info 1
  icq sns info 23ten-uaaaa-aaaaq-aabia-cai
  icq --network ic sns info 1 --format json";

#[derive(Debug, ThisError)]
pub enum SnsCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Host(#[from] SnsHostError),

    #[error("system clock before unix epoch: {0}")]
    Clock(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsListOptions {
    network: String,
    format: OutputFormat,
    source_endpoint: String,
    verbose: bool,
    sort: SnsListSortArg,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsInfoOptions {
    input: String,
    network: String,
    format: OutputFormat,
    source_endpoint: String,
}

pub fn run<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand(sns_command(), args)
        .map_err(|_| SnsCommandError::Usage(usage()))?;

    match command.as_str() {
        "list" => run_sns_list(args),
        "info" => run_sns_info(args),
        _ => unreachable!("sns dispatch command only defines known commands"),
    }
}

fn run_sns_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_list_usage, version_text()) {
        return Ok(());
    }
    let options = SnsListOptions::parse(args)?;
    let format = options.format;
    let request = SnsListRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        verbose: options.verbose,
        sort: options.sort.into(),
    };
    let report = build_sns_list_report(&request)?;
    write_text_or_json(format, &report, sns_list_report_text)
}

fn run_sns_info<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_info_usage, version_text()) {
        return Ok(());
    }
    let options = SnsInfoOptions::parse(args)?;
    let format = options.format;
    let request = SnsInfoRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: current_unix_secs().map_err(SnsCommandError::Clock)?,
        input: options.input,
    };
    let report = build_sns_info_report(&request)?;
    write_text_or_json(format, &report, sns_info_report_text)
}

impl SnsListOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_list_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_list_usage()))?;
        Ok(Self {
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
            verbose: matches.get_flag("verbose"),
            sort: required_typed(&matches, "sort"),
        })
    }
}

impl SnsInfoOptions {
    fn parse<I>(args: I) -> Result<Self, SnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches(sns_info_command(), args)
            .map_err(|_| SnsCommandError::Usage(sns_info_usage()))?;
        Ok(Self {
            input: required_string(&matches, "input"),
            network: required_string(&matches, "network"),
            format: required_typed(&matches, "format"),
            source_endpoint: required_string(&matches, "source-endpoint"),
        })
    }
}

fn sns_command() -> ClapCommand {
    ClapCommand::new("sns")
        .bin_name("icq sns")
        .about("Inspect SNS metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List deployed mainnet SNS instances"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("info").about("Resolve a deployed SNS by list id or root principal"),
        ))
}

fn sns_list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq sns list")
        .about("List deployed mainnet SNS instances")
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance metadata queries"),
        )
        .arg(
            flag_arg("verbose")
                .long("verbose")
                .help("Show full canister IDs in text output"),
        )
        .arg(sort_arg())
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_LIST_HELP_AFTER)
}

fn sns_info_command() -> ClapCommand {
    ClapCommand::new("info")
        .bin_name("icq sns info")
        .about("Resolve a deployed SNS by list id or root principal")
        .disable_help_flag(true)
        .arg(
            value_arg("input")
                .value_name("id|root-principal")
                .required(true)
                .help("SNS list id or root canister principal"),
        )
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_SNS_SOURCE_ENDPOINT)
                .help("IC API endpoint used for SNS-W and governance metadata queries"),
        )
        .arg(internal_network_arg().default_value("ic"))
        .after_help(SNS_INFO_HELP_AFTER)
}

fn usage() -> String {
    render_help(sns_command())
}

fn sns_list_usage() -> String {
    render_help(sns_list_command())
}

fn sns_info_usage() -> String {
    render_help(sns_info_command())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum SnsListSortArg {
    Id,
    Name,
}

impl From<SnsListSortArg> for SnsListSort {
    fn from(value: SnsListSortArg) -> Self {
        match value {
            SnsListSortArg::Id => Self::Id,
            SnsListSortArg::Name => Self::Name,
        }
    }
}

fn sort_arg() -> clap::Arg {
    value_arg("sort")
        .long("sort")
        .value_name("id|name")
        .default_value("id")
        .value_parser(clap::value_parser!(SnsListSortArg))
        .help("Text/JSON row order; ids stay stable by root principal")
}

#[cfg(test)]
mod tests;
