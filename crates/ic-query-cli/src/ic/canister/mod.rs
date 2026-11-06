//! Module: ic::canister
//!
//! Responsibility: parse and dispatch official Dashboard canister commands.
//! Does not own: Dashboard transport, report construction, or text rendering.
//! Boundary: exposes one bounded canister command family to the IC CLI facade.

use super::IcCommandError;
use crate::cli::clap::{parsed_options, with_options};
use crate::cli::common::OutputFormat;
use crate::cli::{
    clap::principal_text_value_parser,
    common::{
        COLLECTION_MODE_LIVE, SourceOutputOptions, collection_help, current_unix_secs,
        write_text_or_json,
    },
};
use clap::{
    ArgMatches, Args, Command as ClapCommand, FromArgMatches, builder::RangedU64ValueParser,
};
use ic_query::ic::DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT;
use ic_query::ic::DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT;
use ic_query::ic::IcCanisterCountRequest;
use ic_query::ic::IcCanisterFilters;
use ic_query::ic::IcCanisterPageRequest;
use ic_query::ic::IcCanisterRequest;
use ic_query::ic::MAX_IC_CANISTER_PAGE_LIMIT;
use ic_query::ic::build_ic_canister_count_report;
use ic_query::ic::build_ic_canister_page_report;
use ic_query::ic::build_ic_canister_report;
use ic_query_text::ic::ic_canister_count_report_text;
use ic_query_text::ic::ic_canister_page_report_text;
use ic_query_text::ic::ic_canister_report_text;

const CANISTER_INFO_HELP_AFTER: &str = "\
Examples:
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
  icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --source-endpoint https://ic-api.internetcomputer.org/api/v3

The official Dashboard API is an off-chain analytics authority. Its response
is not presented as certified IC state or an exact point-in-time snapshot.";
const DEFAULT_CANISTER_PAGE_LIMIT_ARG: &str = "50";

const CANISTER_COUNT_HELP_AFTER: &str = "\
Examples:
  icq ic canister count
  icq ic canister count --has-name true --canister-type ledger --json

This command makes exactly one official Dashboard count request. It does not
fetch canister rows, follow cursors, or create a cache.";

const CANISTER_PAGE_HELP_AFTER: &str = "\
Examples:
  icq ic canister page --limit 25
  icq ic canister page --query ledger --limit 25 --json
  icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25

This command makes exactly one official Dashboard page request. Results are
ordered by canister id, the limit is capped at 100, and returned cursors are
followed only when supplied explicitly to a later command. No cache is used.";

pub(super) async fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches).await,
        Some(("count", matches)) => run_count(matches).await,
        Some(("page", matches)) => run_page(matches).await,
        _ => unreachable!("clap requires a known ic canister subcommand"),
    }
}

async fn run_count(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let (format, request) = count_request(matches, current_unix_secs()?);
    let report = build_ic_canister_count_report(&request).await?;
    write_text_or_json(format, &report, ic_canister_count_report_text)
}

async fn run_page(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let (format, request) = page_request(matches, current_unix_secs()?);
    let report = build_ic_canister_page_report(&request).await?;
    write_text_or_json(format, &report, ic_canister_page_report_text)
}

async fn run_info(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let (format, request) = info_request(matches, current_unix_secs()?);
    let report = build_ic_canister_report(&request).await?;
    write_text_or_json(format, &report, ic_canister_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("canister")
        .about("Inspect deployed canister metadata")
        .subcommand(info_command())
        .subcommand(count_command())
        .subcommand(page_command())
}

fn count_command() -> ClapCommand {
    collection_args(
        with_options::<CanisterFilterOptions>(ClapCommand::new("count"))
            .about("Count canisters through one official Dashboard API request"),
    )
    .after_help(collection_help(
        COLLECTION_MODE_LIVE,
        CANISTER_COUNT_HELP_AFTER,
    ))
}

///
/// CanisterPageOptions
///
/// Typed arguments for the page_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct CanisterPageOptions {
    #[arg(long, value_name = "rows", default_value = DEFAULT_CANISTER_PAGE_LIMIT_ARG, value_parser = RangedU64ValueParser::<u16>::new().range(1..=u64::from(MAX_IC_CANISTER_PAGE_LIMIT)), help = "Maximum rows; one through 100")]
    pub(super) limit: u16,
    #[arg(long, value_name = "canister-id", value_parser = principal_text_value_parser(), conflicts_with = "before", help = "Exclusive forward cursor returned by a prior page")]
    pub(super) after: Option<String>,
    #[arg(long, value_name = "canister-id", value_parser = principal_text_value_parser(), help = "Exclusive backward cursor returned by a prior page")]
    pub(super) before: Option<String>,
}

fn page_command() -> ClapCommand {
    with_options::<CanisterPageOptions>(
        collection_args(
            with_options::<CanisterFilterOptions>(ClapCommand::new("page"))
                .about("Show one bounded official Dashboard canister page"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            CANISTER_PAGE_HELP_AFTER,
        )),
    )
}

fn collection_args(command: ClapCommand) -> ClapCommand {
    with_options::<crate::cli::common::SourceOutputOptions>(command).mut_arg(
        "source-endpoint",
        |arg| {
            arg.default_value(DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API v4 base endpoint")
        },
    )
}

///
/// CanisterFilterOptions
///
/// Clap-owned filter declarations shared by count and page commands.
///

#[derive(Args)]
#[group(skip)]
struct CanisterFilterOptions {
    #[arg(
        long,
        value_name = "true|false",
        help = "Filter by whether a Dashboard name is recorded"
    )]
    has_name: Option<bool>,
    #[arg(long, value_name = "principal", value_parser = principal_text_value_parser(), help = "Filter by Subnet principal")]
    subnet_id: Option<String>,
    #[arg(
        long,
        value_name = "principal",
        value_parser = principal_text_value_parser(),
        help = "Filter by controller principal"
    )]
    controller_id: Option<String>,
    #[arg(
        long = "language",
        value_name = "label",
        value_parser = clap::builder::NonEmptyStringValueParser::new(),
        help = "Filter by a raw Dashboard language label; repeatable"
    )]
    languages: Vec<String>,
    #[arg(
        long = "canister-type",
        value_name = "classification",
        value_parser = clap::builder::NonEmptyStringValueParser::new(),
        help = "Filter by a raw Dashboard canister classification; repeatable"
    )]
    canister_types: Vec<String>,
    #[arg(
        long,
        value_name = "text",
        help = "Dashboard text search; two through 100 characters"
    )]
    query: Option<String>,
}

///
/// CanisterInfoOptions
///
/// Typed arguments for the info_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct CanisterInfoOptions {
    #[arg(id = "canister-id", value_name = "canister-id", value_parser = principal_text_value_parser(), help = "Canister principal")]
    pub(super) canister_id: String,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

fn info_command() -> ClapCommand {
    with_options::<CanisterInfoOptions>(
        ClapCommand::new("info")
            .about("Show one canister from the official Dashboard API")
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                CANISTER_INFO_HELP_AFTER,
            )),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official IC Dashboard API base endpoint")
    })
}

fn canister_filters(matches: &ArgMatches) -> IcCanisterFilters {
    let options = CanisterFilterOptions::from_arg_matches(matches)
        .expect("Clap admitted the canister filter schema");
    IcCanisterFilters {
        has_name: options.has_name,
        subnet_id: options.subnet_id,
        controller_id: options.controller_id,
        languages: options.languages,
        canister_types: options.canister_types,
        query: options.query,
    }
}

fn info_request(matches: &ArgMatches, now_unix_secs: u64) -> (OutputFormat, IcCanisterRequest) {
    let options: CanisterInfoOptions = parsed_options(matches);
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcCanisterRequest::new(dashboard, options.canister_id);
    (format, request)
}

fn count_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcCanisterCountRequest) {
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let mut request = IcCanisterCountRequest::new(dashboard);
    request.filters = canister_filters(matches);

    (format, request)
}

fn page_request(matches: &ArgMatches, now_unix_secs: u64) -> (OutputFormat, IcCanisterPageRequest) {
    let options: CanisterPageOptions = parsed_options(matches);
    let (format, collection) = count_request(matches, now_unix_secs);
    let mut request = IcCanisterPageRequest::new(collection.dashboard);
    request.filters = collection.filters;
    request.limit = options.limit;

    request.after = options.after;
    request.before = options.before;
    (format, request)
}

#[cfg(test)]
mod tests;
