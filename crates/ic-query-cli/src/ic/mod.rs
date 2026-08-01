//! Module: ic
//!
//! Responsibility: parse and dispatch official IC Dashboard command families.
//! Does not own: REST transport, report construction, or text rendering.
//! Boundary: exposes bounded live Dashboard command wiring to the top-level CLI.

#[cfg(test)]
use crate::cli::clap::render_help;
use crate::cli::{
    clap::{required_string, required_typed, string_option, typed_option, value_arg},
    common::{
        COLLECTION_MODE_LIVE, CurrentUnixSecsError, OutputFormat, collection_help,
        current_unix_secs, json_arg, output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgAction, ArgMatches, Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::{
    DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT, DEFAULT_IC_DAILY_STATS_WINDOW_SECS,
    DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
    DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT, DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
    DEFAULT_IC_METRIC_STEP_SECS, DEFAULT_IC_METRIC_WINDOW_SECS, IcBoundaryNodeDataCentersRequest,
    IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageRequest, IcCanisterRequest,
    IcDailyStatsQuery, IcDailyStatsRequest, IcHostError, IcMetricKind, IcMetricQuery,
    IcMetricRequest, MAX_IC_CANISTER_PAGE_LIMIT, MAX_IC_METRIC_STEP_SECS,
    MIN_IC_DAILY_STATS_TIMESTAMP, MIN_IC_METRIC_TIMESTAMP,
    build_ic_boundary_node_data_centers_report, build_ic_canister_count_report,
    build_ic_canister_page_report, build_ic_canister_report, build_ic_daily_stats_report,
    build_ic_metric_report, ic_boundary_node_data_centers_report_text,
    ic_canister_count_report_text, ic_canister_page_report_text, ic_canister_report_text,
    ic_daily_stats_report_text, ic_metric_report_text,
};
#[cfg(test)]
use std::ffi::OsString;
use std::io;
use thiserror::Error as ThisError;

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

const METRICS_HELP_AFTER: &str = "\
Examples:
  icq ic metrics instruction-rate
  icq ic metrics cycle-burn-rate --start 1700000000 --end 1700003600 --step 300
  icq ic metrics ic-node-count --json

This command makes exactly one official Dashboard Metrics API request for an
explicitly bounded window. It never follows up, paginates, or creates a cache.
The default is the preceding hour at a five-minute step, and every request is
capped at 1000 observations per returned series. Values are preserved as the
raw value strings returned by this off-chain, non-certified API.";

const BOUNDARY_NODE_DATA_CENTERS_HELP_AFTER: &str = "\
Examples:
  icq ic network boundary-node-data-centers
  icq ic network boundary-node-data-centers --json

This command makes exactly one official Dashboard v4 request for the complete
boundary-node data-center resource. It does not issue per-location follow-up
calls or create a cache. Rows preserve the API's raw owner, region, coordinate,
and node-count strings, including locations that currently report zero nodes.
The Dashboard response is off-chain and non-certified.";

const DAILY_STATS_HELP_AFTER: &str = "\
Examples:
  icq ic network daily-stats
  icq ic network daily-stats --start 1784937600 --end 1785542400
  icq ic network daily-stats --json

This command makes exactly one official Dashboard v3 request for an explicitly
bounded daily network-activity window. The default is the preceding seven days,
and every request is capped at one year and 366 returned rows. It never follows
up, paginates, or creates a cache. Rate values remain the raw strings returned
by this off-chain, non-certified API.";

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

pub fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("canister", matches)) => run_canister(matches),
        Some(("metrics", matches)) => run_metrics(matches),
        Some(("network", matches)) => run_network(matches),
        _ => unreachable!("clap requires a known ic subcommand"),
    }
}

fn run_network(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("boundary-node-data-centers", matches)) => run_boundary_node_data_centers(matches),
        Some(("daily-stats", matches)) => run_daily_stats(matches),
        _ => unreachable!("clap requires a known ic network subcommand"),
    }
}

fn run_boundary_node_data_centers(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = NetworkReportOptions::from_matches(matches);
    let request =
        IcBoundaryNodeDataCentersRequest::new(options.source_endpoint, current_unix_secs()?);
    let report = build_ic_boundary_node_data_centers_report(&request)?;
    write_text_or_json(
        options.format,
        &report,
        ic_boundary_node_data_centers_report_text,
    )
}

fn run_daily_stats(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = DailyStatsOptions::from_matches(matches);
    let now_unix_secs = current_unix_secs()?;
    let end_unix_secs = options.end_unix_secs.unwrap_or(now_unix_secs);
    let start_unix_secs = options
        .start_unix_secs
        .unwrap_or_else(|| end_unix_secs.saturating_sub(DEFAULT_IC_DAILY_STATS_WINDOW_SECS));
    let request = IcDailyStatsRequest::new(
        options.source_endpoint,
        now_unix_secs,
        IcDailyStatsQuery::new(start_unix_secs, end_unix_secs),
    );
    let report = build_ic_daily_stats_report(&request)?;
    write_text_or_json(options.format, &report, ic_daily_stats_report_text)
}

fn run_metrics(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = MetricOptions::from_matches(matches);
    let now_unix_secs = current_unix_secs()?;
    let end_unix_secs = options.end_unix_secs.unwrap_or(now_unix_secs);
    let start_unix_secs = options
        .start_unix_secs
        .unwrap_or_else(|| end_unix_secs.saturating_sub(DEFAULT_IC_METRIC_WINDOW_SECS));
    let request = IcMetricRequest::new(
        options.source_endpoint,
        now_unix_secs,
        IcMetricQuery::new(
            options.metric,
            start_unix_secs,
            end_unix_secs,
            options.step_secs,
        ),
    );
    let report = build_ic_metric_report(&request)?;
    write_text_or_json(options.format, &report, ic_metric_report_text)
}

fn run_canister(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_canister_info(matches),
        Some(("count", matches)) => run_canister_count(matches),
        Some(("page", matches)) => run_canister_page(matches),
        _ => unreachable!("clap requires a known ic canister subcommand"),
    }
}

fn run_canister_count(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = CanisterCollectionOptions::from_matches(matches);
    let request = IcCanisterCountRequest::new(options.source_endpoint, current_unix_secs()?)
        .with_filters(options.filters);
    let report = build_ic_canister_count_report(&request)?;
    write_text_or_json(options.format, &report, ic_canister_count_report_text)
}

fn run_canister_page(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = CanisterPageOptions::from_matches(matches);
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

fn run_canister_info(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = CanisterInfoOptions::from_matches(matches);
    let request = IcCanisterRequest::new(
        options.source_endpoint,
        current_unix_secs()?,
        options.canister_id,
    );
    let report = build_ic_canister_report(&request)?;
    write_text_or_json(options.format, &report, ic_canister_report_text)
}

pub fn command() -> ClapCommand {
    ClapCommand::new("ic")
        .bin_name("icq ic")
        .about("Inspect official IC Dashboard data")
        .subcommand_required(true)
        .subcommand(canister_command())
        .subcommand(metrics_command())
        .subcommand(network_command())
}

fn network_command() -> ClapCommand {
    ClapCommand::new("network")
        .bin_name("icq ic network")
        .about("Inspect bounded official Dashboard network analytics")
        .subcommand_required(true)
        .subcommand(boundary_node_data_centers_command())
        .subcommand(daily_stats_command())
}

fn boundary_node_data_centers_command() -> ClapCommand {
    ClapCommand::new("boundary-node-data-centers")
        .bin_name("icq ic network boundary-node-data-centers")
        .about("List official Dashboard boundary-node data-center aggregates")
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API v4 base endpoint"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            BOUNDARY_NODE_DATA_CENTERS_HELP_AFTER,
        ))
}

fn daily_stats_command() -> ClapCommand {
    ClapCommand::new("daily-stats")
        .bin_name("icq ic network daily-stats")
        .about("Query bounded official Dashboard daily network activity")
        .arg(
            value_arg("start")
                .long("start")
                .value_name("unix-seconds")
                .value_parser(
                    RangedU64ValueParser::<u64>::new().range(MIN_IC_DAILY_STATS_TIMESTAMP..),
                )
                .help("Inclusive start; defaults to seven days before end"),
        )
        .arg(
            value_arg("end")
                .long("end")
                .value_name("unix-seconds")
                .value_parser(
                    RangedU64ValueParser::<u64>::new().range(MIN_IC_DAILY_STATS_TIMESTAMP..),
                )
                .help("Inclusive end; defaults to the current time"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API v3 base endpoint"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            DAILY_STATS_HELP_AFTER,
        ))
}

fn metrics_command() -> ClapCommand {
    ClapCommand::new("metrics")
        .bin_name("icq ic metrics")
        .about("Query one bounded official Dashboard network metric time series")
        .arg(
            value_arg("metric")
                .required(true)
                .value_name("metric")
                .value_parser(IcMetricKind::all().map(IcMetricKind::as_str))
                .help("Official Dashboard metric path name"),
        )
        .arg(
            value_arg("start")
                .long("start")
                .value_name("unix-seconds")
                .value_parser(RangedU64ValueParser::<u64>::new().range(MIN_IC_METRIC_TIMESTAMP..))
                .help("Inclusive start; defaults to one hour before end"),
        )
        .arg(
            value_arg("end")
                .long("end")
                .value_name("unix-seconds")
                .value_parser(RangedU64ValueParser::<u64>::new().range(MIN_IC_METRIC_TIMESTAMP..))
                .help("Inclusive end; defaults to the current time"),
        )
        .arg(
            value_arg("step")
                .long("step")
                .value_name("seconds")
                .value_parser(
                    RangedU64ValueParser::<u32>::new()
                        .range(1..=u64::from(MAX_IC_METRIC_STEP_SECS)),
                )
                .help("Observation step in seconds; defaults to 300"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT)
                .help("Official IC Dashboard Metrics API base endpoint"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, METRICS_HELP_AFTER))
}

fn canister_command() -> ClapCommand {
    ClapCommand::new("canister")
        .bin_name("icq ic canister")
        .about("Inspect deployed canister metadata")
        .subcommand_required(true)
        .subcommand(canister_info_command())
        .subcommand(canister_count_command())
        .subcommand(canister_page_command())
}

fn canister_count_command() -> ClapCommand {
    canister_collection_args(
        ClapCommand::new("count")
            .bin_name("icq ic canister count")
            .about("Count canisters through one official Dashboard API request"),
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
            .about("Show one bounded official Dashboard canister page"),
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
            .help("Exclusive backward cursor returned by a prior page"),
    )
    .after_help(collection_help(
        COLLECTION_MODE_LIVE,
        CANISTER_PAGE_HELP_AFTER,
    ))
}

fn canister_collection_args(command: ClapCommand) -> ClapCommand {
    canister_filter_args(command).arg(json_arg()).arg(
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
        .arg(
            value_arg("canister-id")
                .required(true)
                .value_name("canister-id")
                .help("Canister principal"),
        )
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT)
                .help("Official IC Dashboard API base endpoint"),
        )
        .after_help(collection_help(
            COLLECTION_MODE_LIVE,
            CANISTER_INFO_HELP_AFTER,
        ))
}

#[cfg(test)]
fn usage() -> String {
    render_help(command())
}

#[cfg(test)]
fn canister_usage() -> String {
    render_help(canister_command())
}

#[cfg(test)]
fn canister_info_usage() -> String {
    render_help(canister_info_command())
}

#[cfg(test)]
fn canister_count_usage() -> String {
    render_help(canister_count_command())
}

#[cfg(test)]
fn canister_page_usage() -> String {
    render_help(canister_page_command())
}

#[cfg(test)]
fn metrics_usage() -> String {
    render_help(metrics_command())
}

#[cfg(test)]
fn network_usage() -> String {
    render_help(network_command())
}

#[cfg(test)]
fn boundary_node_data_centers_usage() -> String {
    render_help(boundary_node_data_centers_command())
}

#[cfg(test)]
fn daily_stats_usage() -> String {
    render_help(daily_stats_command())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanisterInfoOptions {
    canister_id: String,
    format: OutputFormat,
    source_endpoint: String,
}

impl CanisterInfoOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            canister_id: required_string(matches, "canister-id"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }

    #[cfg(test)]
    fn parse<I>(args: I) -> Result<Self, IcCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::cli::clap::parse_matches_or_usage(canister_info_command(), args)
            .map_err(IcCommandError::Usage)?;
        Ok(Self::from_matches(&matches))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MetricOptions {
    metric: IcMetricKind,
    start_unix_secs: Option<u64>,
    end_unix_secs: Option<u64>,
    step_secs: u32,
    format: OutputFormat,
    source_endpoint: String,
}

impl MetricOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        let metric = required_string(matches, "metric")
            .parse()
            .expect("clap restricts official metric names");
        Self {
            metric,
            start_unix_secs: typed_option(matches, "start"),
            end_unix_secs: typed_option(matches, "end"),
            step_secs: typed_option(matches, "step").unwrap_or(DEFAULT_IC_METRIC_STEP_SECS),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }

    #[cfg(test)]
    fn parse<I>(args: I) -> Result<Self, IcCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::cli::clap::parse_matches_or_usage(metrics_command(), args)
            .map_err(IcCommandError::Usage)?;
        Ok(Self::from_matches(&matches))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NetworkReportOptions {
    format: OutputFormat,
    source_endpoint: String,
}

impl NetworkReportOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }

    #[cfg(test)]
    fn parse_boundary_node_data_centers<I>(args: I) -> Result<Self, IcCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            crate::cli::clap::parse_matches_or_usage(boundary_node_data_centers_command(), args)
                .map_err(IcCommandError::Usage)?;
        Ok(Self::from_matches(&matches))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DailyStatsOptions {
    start_unix_secs: Option<u64>,
    end_unix_secs: Option<u64>,
    format: OutputFormat,
    source_endpoint: String,
}

impl DailyStatsOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            start_unix_secs: typed_option(matches, "start"),
            end_unix_secs: typed_option(matches, "end"),
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }

    #[cfg(test)]
    fn parse<I>(args: I) -> Result<Self, IcCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::cli::clap::parse_matches_or_usage(daily_stats_command(), args)
            .map_err(IcCommandError::Usage)?;
        Ok(Self::from_matches(&matches))
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
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[cfg(test)]
fn parse_canister_count_options<I>(args: I) -> Result<CanisterCollectionOptions, IcCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let matches = crate::cli::clap::parse_matches_or_usage(canister_count_command(), args)
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
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            collection: CanisterCollectionOptions::from_matches(matches),
            limit: required_typed(matches, "limit"),
            after: string_option(matches, "after"),
            before: string_option(matches, "before"),
        }
    }

    #[cfg(test)]
    fn parse<I>(args: I) -> Result<Self, IcCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::cli::clap::parse_matches_or_usage(canister_page_command(), args)
            .map_err(IcCommandError::Usage)?;
        Ok(Self::from_matches(&matches))
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
