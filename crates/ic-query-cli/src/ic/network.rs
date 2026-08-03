//! Module: ic::network
//!
//! Responsibility: parse and dispatch bounded official Dashboard network resources.
//! Does not own: Dashboard transport, report construction, or text rendering.
//! Boundary: exposes the network command family to the IC CLI facade.

use super::IcCommandError;
#[cfg(test)]
use super::parse_test_options;
use crate::cli::{
    clap::{required_string, typed_option, value_arg},
    common::{
        COLLECTION_MODE_LIVE, OutputFormat, collection_help, current_unix_secs, json_arg,
        output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand, builder::RangedU64ValueParser};
use ic_query::ic::{
    DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT, DEFAULT_IC_DAILY_STATS_WINDOW_SECS,
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcBoundaryNodeDataCentersRequest, IcDailyStatsQuery,
    IcDailyStatsRequest, MIN_IC_DAILY_STATS_TIMESTAMP, build_ic_boundary_node_data_centers_report,
    build_ic_daily_stats_report, ic_boundary_node_data_centers_report_text,
    ic_daily_stats_report_text,
};

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

pub(super) fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
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

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("network")
        .bin_name("icq ic network")
        .about("Inspect bounded official Dashboard network analytics")
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;

    #[test]
    fn usage_discloses_live_dashboard_authority_and_bounds() {
        let network = render_help(command());
        let boundary_nodes = render_help(boundary_node_data_centers_command());
        let daily_stats = render_help(daily_stats_command());

        assert!(network.contains("Usage: icq ic network [COMMAND]"));
        assert!(network.contains("boundary-node-data-centers"));
        assert!(network.contains("daily-stats"));
        assert!(
            boundary_nodes.contains("Usage: icq ic network boundary-node-data-centers [OPTIONS]")
        );
        assert!(boundary_nodes.contains("exactly one official Dashboard v4 request"));
        assert!(boundary_nodes.contains("locations that currently report zero nodes"));
        assert!(daily_stats.contains("Usage: icq ic network daily-stats [OPTIONS]"));
        assert!(daily_stats.contains("exactly one official Dashboard v3 request"));
        assert!(daily_stats.contains("capped at one year and 366 returned rows"));
    }

    #[test]
    fn boundary_node_data_center_options_preserve_format_and_endpoint() {
        let options = parse_test_options(
            boundary_node_data_centers_command(),
            &["--json", "--source-endpoint", "https://example.com/api/v4"],
            NetworkReportOptions::from_matches,
        )
        .expect("boundary-node options");

        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.source_endpoint, "https://example.com/api/v4");
    }

    #[test]
    fn daily_stats_options_preserve_bounds_format_and_endpoint() {
        let options = parse_test_options(
            daily_stats_command(),
            &[
                "--start",
                "1784937600",
                "--end",
                "1785542400",
                "--json",
                "--source-endpoint",
                "https://example.com/api/v3",
            ],
            DailyStatsOptions::from_matches,
        )
        .expect("daily-statistics options");

        assert_eq!(options.start_unix_secs, Some(1_784_937_600));
        assert_eq!(options.end_unix_secs, Some(1_785_542_400));
        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.source_endpoint, "https://example.com/api/v3");
    }

    #[test]
    fn daily_stats_options_use_live_bounded_defaults() {
        let options =
            parse_test_options(daily_stats_command(), &[], DailyStatsOptions::from_matches)
                .expect("default daily-statistics options");

        assert_eq!(options.start_unix_secs, None);
        assert_eq!(options.end_unix_secs, None);
        assert_eq!(options.format, OutputFormat::Text);
        assert_eq!(
            options.source_endpoint,
            DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT
        );
    }
}
