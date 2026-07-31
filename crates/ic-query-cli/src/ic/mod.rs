//! Module: ic
//!
//! Responsibility: parse and dispatch official IC Dashboard command families.
//! Does not own: REST transport, report construction, or text rendering.
//! Boundary: exposes live-only canister command wiring to the top-level CLI.

use crate::{
    cli::{
        clap::{
            parse_matches_or_usage, parse_required_subcommand_or_usage, passthrough_subcommand,
            render_help, required_string, required_typed, string_option, typed_option, value_arg,
        },
        common::{
            COLLECTION_MODE_LIVE, CurrentUnixSecsError, OutputFormat, collection_help,
            current_unix_secs, format_arg, source_endpoint_arg, write_text_or_json,
        },
        help::collect_args_or_print_help_or_version,
    },
    version_text,
};
use clap::{ArgAction, Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT, DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
    IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageRequest, IcCanisterRequest,
    IcHostError, MAX_IC_CANISTER_PAGE_LIMIT, build_ic_canister_count_report,
    build_ic_canister_page_report, build_ic_canister_report, ic_canister_count_report_text,
    ic_canister_page_report_text, ic_canister_report_text,
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
const DEFAULT_CANISTER_PAGE_LIMIT_ARG: &str = "50";

const CANISTER_COUNT_HELP_AFTER: &str = "\
Examples:
  icq ic canister count
  icq ic canister count --has-name true --canister-type ledger --format json

This command makes exactly one official Dashboard count request. It does not
fetch canister rows, follow cursors, or create a cache.";

const CANISTER_PAGE_HELP_AFTER: &str = "\
Examples:
  icq ic canister page --limit 25
  icq ic canister page --query ledger --limit 25 --format json
  icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25

This command makes exactly one official Dashboard page request. Results are
ordered by canister id, the limit is capped at 100, and returned cursors are
followed only when supplied explicitly to a later command. No cache is used.";

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
        "count" => run_canister_count(args),
        "page" => run_canister_page(args),
        _ => unreachable!("ic canister dispatch command only defines known commands"),
    }
}

fn run_canister_count<I>(args: I) -> Result<(), IcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, canister_count_usage) else {
        return Ok(());
    };
    let options = parse_canister_count_options(args)?;
    let request = IcCanisterCountRequest::new(options.source_endpoint, current_unix_secs()?)
        .with_filters(options.filters);
    let report = build_ic_canister_count_report(&request)?;
    write_text_or_json(options.format, &report, ic_canister_count_report_text)
}

fn run_canister_page<I>(args: I) -> Result<(), IcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, canister_page_usage) else {
        return Ok(());
    };
    let options = CanisterPageOptions::parse(args)?;
    let mut request =
        IcCanisterPageRequest::new(options.collection.source_endpoint, current_unix_secs()?)
            .with_filters(options.collection.filters)
            .with_limit(options.limit);
    request.after = options.after;
    request.before = options.before;
    let report = build_ic_canister_page_report(&request)?;
    write_text_or_json(
        options.collection.format,
        &report,
        ic_canister_page_report_text,
    )
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
        .subcommand(passthrough_subcommand(
            ClapCommand::new("count").about("Count canisters through one Dashboard API request"),
        ))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("page").about("Show one bounded Dashboard canister page"),
        ))
}

fn canister_count_command() -> ClapCommand {
    canister_collection_args(
        ClapCommand::new("count")
            .bin_name("icq ic canister count")
            .about("Count canisters through one official Dashboard API request")
            .disable_help_flag(true),
    )
    .after_help(collection_help(
        COLLECTION_MODE_LIVE,
        CANISTER_COUNT_HELP_AFTER,
    ))
}

fn canister_page_command() -> ClapCommand {
    canister_collection_args(
        ClapCommand::new("page")
            .bin_name("icq ic canister page")
            .about("Show one bounded official Dashboard canister page")
            .disable_help_flag(true),
    )
    .arg(
        value_arg("limit")
            .long("limit")
            .value_name("rows")
            .default_value(DEFAULT_CANISTER_PAGE_LIMIT_ARG)
            .value_parser(
                RangedU64ValueParser::<u16>::new().range(1..=u64::from(MAX_IC_CANISTER_PAGE_LIMIT)),
            )
            .help("Maximum rows; one through 100"),
    )
    .arg(
        value_arg("after")
            .long("after")
            .value_name("canister-id")
            .conflicts_with("before")
            .help("Exclusive forward cursor returned by a prior page"),
    )
    .arg(
        value_arg("before")
            .long("before")
            .value_name("canister-id")
            .conflicts_with("after")
            .help("Exclusive backward cursor returned by a prior page"),
    )
    .after_help(collection_help(
        COLLECTION_MODE_LIVE,
        CANISTER_PAGE_HELP_AFTER,
    ))
}

fn canister_collection_args(command: ClapCommand) -> ClapCommand {
    canister_filter_args(command).arg(format_arg()).arg(
        source_endpoint_arg(DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API v4 base endpoint"),
    )
}

fn canister_filter_args(command: ClapCommand) -> ClapCommand {
    command
        .arg(
            value_arg("has-name")
                .long("has-name")
                .value_name("true|false")
                .value_parser(clap::value_parser!(bool))
                .help("Filter by whether a Dashboard name is recorded"),
        )
        .arg(
            value_arg("subnet-id")
                .long("subnet-id")
                .value_name("principal")
                .help("Filter by Subnet principal"),
        )
        .arg(
            value_arg("controller-id")
                .long("controller-id")
                .value_name("principal")
                .help("Filter by controller principal"),
        )
        .arg(
            value_arg("language")
                .long("language")
                .value_name("label")
                .action(ArgAction::Append)
                .help("Filter by a raw Dashboard language label; repeatable"),
        )
        .arg(
            value_arg("canister-type")
                .long("canister-type")
                .value_name("classification")
                .action(ArgAction::Append)
                .help("Filter by a raw Dashboard canister classification; repeatable"),
        )
        .arg(
            value_arg("query")
                .long("query")
                .value_name("text")
                .help("Dashboard text search; two through 100 characters"),
        )
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

fn canister_count_usage() -> String {
    render_help(canister_count_command())
}

fn canister_page_usage() -> String {
    render_help(canister_page_command())
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanisterCollectionOptions {
    filters: IcCanisterFilters,
    format: OutputFormat,
    source_endpoint: String,
}

impl CanisterCollectionOptions {
    fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            filters: canister_filters(matches),
            format: required_typed(matches, "format"),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

fn parse_canister_count_options<I>(args: I) -> Result<CanisterCollectionOptions, IcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let matches = parse_matches_or_usage(canister_count_command(), args, canister_count_usage)
        .map_err(IcCommandError::Usage)?;
    Ok(CanisterCollectionOptions::from_matches(&matches))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanisterPageOptions {
    collection: CanisterCollectionOptions,
    limit: u16,
    after: Option<String>,
    before: Option<String>,
}

impl CanisterPageOptions {
    fn parse<I>(args: I) -> Result<Self, IcCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_matches_or_usage(canister_page_command(), args, canister_page_usage)
            .map_err(IcCommandError::Usage)?;
        Ok(Self {
            collection: CanisterCollectionOptions::from_matches(&matches),
            limit: required_typed(&matches, "limit"),
            after: string_option(&matches, "after"),
            before: string_option(&matches, "before"),
        })
    }
}

fn canister_filters(matches: &clap::ArgMatches) -> IcCanisterFilters {
    IcCanisterFilters {
        has_name: typed_option(matches, "has-name"),
        subnet_id: string_option(matches, "subnet-id"),
        controller_id: string_option(matches, "controller-id"),
        languages: repeated_strings(matches, "language"),
        canister_types: repeated_strings(matches, "canister-type"),
        query: string_option(matches, "query"),
    }
}

fn repeated_strings(matches: &clap::ArgMatches, id: &str) -> Vec<String> {
    matches
        .get_many::<String>(id)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
