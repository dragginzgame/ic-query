pub mod report;

use crate::{
    cli::{
        clap::{
            flag_arg, parse_matches, parse_required_subcommand, passthrough_subcommand,
            render_help, required_string, required_typed, value_arg,
        },
        globals::internal_network_arg,
        help::print_help_or_version,
    },
    output::{write_pretty_json, write_text},
    sns::report::{
        DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsInfoRequest, SnsListRequest,
        build_sns_info_report, build_sns_list_report, sns_info_report_text, sns_list_report_text,
    },
    version_text,
};
use clap::{Command as ClapCommand, ValueEnum};
use serde::Serialize;
use std::{
    ffi::OsString,
    io,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error as ThisError;

const SNS_LIST_HELP_AFTER: &str = "\
Examples:
  icq sns list
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsListOptions {
    network: String,
    format: OutputFormat,
    source_endpoint: String,
    verbose: bool,
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
        now_unix_secs: now_unix_secs()?,
        verbose: options.verbose,
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
        now_unix_secs: now_unix_secs()?,
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

fn write_text_or_json<T>(
    format: OutputFormat,
    report: &T,
    render_text: impl FnOnce(&T) -> String,
) -> Result<(), SnsCommandError>
where
    T: Serialize,
{
    match format {
        OutputFormat::Text => {
            let text = render_text(report);
            write_text::<SnsCommandError>(None, &text)
        }
        OutputFormat::Json => write_pretty_json(None, report),
    }
}

fn now_unix_secs() -> Result<u64, SnsCommandError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| SnsCommandError::Clock(err.to_string()))
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

fn format_arg() -> clap::Arg {
    value_arg("format")
        .long("format")
        .value_name("text|json")
        .default_value("text")
        .value_parser(clap::value_parser!(OutputFormat))
        .help("Output format; defaults to text")
}

fn source_endpoint_arg(default_source_endpoint: &'static str) -> clap::Arg {
    value_arg("source-endpoint")
        .long("source-endpoint")
        .value_name("url")
        .default_value(default_source_endpoint)
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

#[cfg(test)]
mod tests;
