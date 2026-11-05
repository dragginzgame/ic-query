//! Module: ic::metrics
//!
//! Responsibility: parse and run bounded official Dashboard metric queries.
//! Does not own: Dashboard transport, report construction, or text rendering.
//! Boundary: exposes the metric command to the IC CLI facade.

use super::IcCommandError;
use crate::cli::common::OutputFormat;
use crate::cli::{
    clap::{required_typed, typed_option, value_arg},
    common::{
        COLLECTION_MODE_LIVE, SourceOutputOptions, collection_help, current_unix_secs, json_arg,
        resolve_time_bounds, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{
    ArgMatches, Command as ClapCommand,
    builder::{PossibleValuesParser, RangedU64ValueParser, TypedValueParser},
};
use ic_query::ic::DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT;
use ic_query::ic::DEFAULT_IC_METRIC_STEP_SECS;
use ic_query::ic::DEFAULT_IC_METRIC_WINDOW_SECS;
use ic_query::ic::IcMetricKind;
use ic_query::ic::IcMetricQuery;
use ic_query::ic::IcMetricRequest;
use ic_query::ic::MAX_IC_METRIC_STEP_SECS;
use ic_query::ic::MIN_IC_METRIC_TIMESTAMP;
use ic_query::ic::build_ic_metric_report;
use ic_query_text::ic::ic_metric_report_text;

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

pub(super) async fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let (format, request) = metric_request(matches, current_unix_secs()?);
    let report = build_ic_metric_report(&request).await?;
    write_text_or_json(format, &report, ic_metric_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("metrics")
        .about("Query one bounded official Dashboard network metric time series")
        .arg(
            value_arg("metric")
                .required(true)
                .value_name("metric")
                .value_parser(
                    PossibleValuesParser::new(IcMetricKind::all().map(IcMetricKind::as_str))
                        .try_map(|value| value.parse::<IcMetricKind>()),
                )
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
                .default_value(DEFAULT_IC_METRIC_STEP_SECS.to_string())
                .hide_default_value(true)
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

fn metric_request(matches: &ArgMatches, now_unix_secs: u64) -> (OutputFormat, IcMetricRequest) {
    let (start_unix_secs, end_unix_secs) = resolve_time_bounds(
        typed_option(matches, "start"),
        typed_option(matches, "end"),
        now_unix_secs,
        DEFAULT_IC_METRIC_WINDOW_SECS,
        0,
    );
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(now_unix_secs);
    let request = IcMetricRequest::new(
        dashboard,
        IcMetricQuery::new(
            required_typed(matches, "metric"),
            start_unix_secs,
            end_unix_secs,
            required_typed(matches, "step"),
        ),
    );
    (format, request)
}

#[cfg(test)]
mod tests;
