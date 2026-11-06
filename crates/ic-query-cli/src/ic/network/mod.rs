//! Module: ic::network
//!
//! Responsibility: parse and dispatch bounded official Dashboard network resources.
//! Does not own: Dashboard transport, report construction, or text rendering.
//! Boundary: exposes the network command family to the IC CLI facade.

use super::IcCommandError;
use crate::cli::clap::{parsed_options, with_source_options};
use crate::cli::common::OutputFormat;
use crate::cli::common::{
    COLLECTION_MODE_LIVE, SourceOutputOptions, collection_help, current_unix_secs,
    resolve_time_bounds, write_text_or_json,
};
use clap::{ArgMatches, Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT;
use ic_query::ic::DEFAULT_IC_DAILY_STATS_WINDOW_SECS;
use ic_query::ic::DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT;
use ic_query::ic::IcDailyStatsQuery;
use ic_query::ic::IcDailyStatsRequest;
use ic_query::ic::MIN_IC_DAILY_STATS_TIMESTAMP;
use ic_query::ic::build_ic_boundary_node_data_centers_report;
use ic_query::ic::build_ic_daily_stats_report;
use ic_query_text::ic::ic_boundary_node_data_centers_report_text;
use ic_query_text::ic::ic_daily_stats_report_text;

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

pub(super) async fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("boundary-node-data-centers", matches)) => {
            run_boundary_node_data_centers(matches).await
        }
        Some(("daily-stats", matches)) => run_daily_stats(matches).await,
        _ => unreachable!("clap requires a known ic network subcommand"),
    }
}

async fn run_boundary_node_data_centers(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = SourceOutputOptions::from_matches(matches);
    let (format, request) = options.into_dashboard_request(current_unix_secs()?);
    let report = build_ic_boundary_node_data_centers_report(&request).await?;
    write_text_or_json(format, &report, ic_boundary_node_data_centers_report_text)
}

async fn run_daily_stats(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let (format, request) = daily_stats_request(matches, current_unix_secs()?);
    let report = build_ic_daily_stats_report(&request).await?;
    write_text_or_json(format, &report, ic_daily_stats_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("network")
        .about("Inspect bounded official Dashboard network analytics")
        .subcommand(boundary_node_data_centers_command())
        .subcommand(daily_stats_command())
}

fn boundary_node_data_centers_command() -> ClapCommand {
    with_source_options::<crate::cli::common::SourceOutputOptions>(
        ClapCommand::new("boundary-node-data-centers")
            .about("List official Dashboard boundary-node data-center aggregates")
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                BOUNDARY_NODE_DATA_CENTERS_HELP_AFTER,
            )),
        DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT,
        "Official IC Dashboard API v4 base endpoint",
    )
}

///
/// DailyStatsOptions
///
/// Typed arguments for the daily_stats_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct DailyStatsOptions {
    #[arg(long, value_name = "unix-seconds", value_parser = RangedU64ValueParser::<u64>::new().range(MIN_IC_DAILY_STATS_TIMESTAMP..), help = "Inclusive start; defaults to seven days before end")]
    pub(super) start: Option<u64>,
    #[arg(long, value_name = "unix-seconds", value_parser = RangedU64ValueParser::<u64>::new().range(MIN_IC_DAILY_STATS_TIMESTAMP..), help = "Inclusive end; defaults to the current time")]
    pub(super) end: Option<u64>,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

fn daily_stats_command() -> ClapCommand {
    with_source_options::<DailyStatsOptions>(
        ClapCommand::new("daily-stats")
            .about("Query bounded official Dashboard daily network activity")
            .after_help(collection_help(
                COLLECTION_MODE_LIVE,
                DAILY_STATS_HELP_AFTER,
            )),
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        "Official IC Dashboard API v3 base endpoint",
    )
}

fn daily_stats_request(
    matches: &ArgMatches,
    now_unix_secs: u64,
) -> (OutputFormat, IcDailyStatsRequest) {
    let options: DailyStatsOptions = parsed_options(matches);
    let (start_unix_secs, end_unix_secs) = resolve_time_bounds(
        options.start,
        options.end,
        now_unix_secs,
        DEFAULT_IC_DAILY_STATS_WINDOW_SECS,
        0,
    );
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcDailyStatsRequest::new(
        dashboard,
        IcDailyStatsQuery::new(start_unix_secs, end_unix_secs),
    );
    (format, request)
}

#[cfg(test)]
mod tests;
