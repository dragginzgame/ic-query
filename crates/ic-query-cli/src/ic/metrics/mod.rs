//! Module: ic::metrics
//!
//! Responsibility: parse and run bounded official Dashboard metric queries.
//! Does not own: Dashboard transport, report construction, or text rendering.
//! Boundary: exposes the metric command to the IC CLI facade.

use super::IcCommandError;
use crate::cli::clap::{parsed_options, with_source_options};
use crate::cli::common::OutputFormat;
use crate::cli::common::{
    COLLECTION_MODE_LIVE, collection_help, current_unix_secs, resolve_time_bounds,
    write_text_or_json,
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

///
/// MetricOptions
///
/// Typed arguments for the command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct MetricOptions {
    #[arg(value_name = "metric", value_parser = PossibleValuesParser::new(IcMetricKind::all().map(IcMetricKind::as_str)) .try_map(|value| value.parse::<IcMetricKind>()), help = "Official Dashboard metric path name")]
    pub(super) metric: IcMetricKind,
    #[arg(long, value_name = "unix-seconds", value_parser = RangedU64ValueParser::<u64>::new().range(MIN_IC_METRIC_TIMESTAMP..), help = "Inclusive start; defaults to one hour before end")]
    pub(super) start: Option<u64>,
    #[arg(long, value_name = "unix-seconds", value_parser = RangedU64ValueParser::<u64>::new().range(MIN_IC_METRIC_TIMESTAMP..), help = "Inclusive end; defaults to the current time")]
    pub(super) end: Option<u64>,
    #[arg(long, value_name = "seconds", default_value = DEFAULT_IC_METRIC_STEP_SECS.to_string(), hide_default_value = true, value_parser = RangedU64ValueParser::<u32>::new() .range(1..=u64::from(MAX_IC_METRIC_STEP_SECS)), help = "Observation step in seconds; defaults to 300")]
    pub(super) step: u32,
    #[command(flatten)]
    pub(super) source: crate::cli::common::SourceOutputOptions,
}

pub(super) fn command() -> ClapCommand {
    with_source_options::<MetricOptions>(
        ClapCommand::new("metrics")
            .about("Query one bounded official Dashboard network metric time series")
            .after_help(collection_help(COLLECTION_MODE_LIVE, METRICS_HELP_AFTER)),
        DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT,
        "Official IC Dashboard Metrics API base endpoint",
    )
}

fn metric_request(matches: &ArgMatches, now_unix_secs: u64) -> (OutputFormat, IcMetricRequest) {
    let options: MetricOptions = parsed_options(matches);
    let (start_unix_secs, end_unix_secs) = resolve_time_bounds(
        options.start,
        options.end,
        now_unix_secs,
        DEFAULT_IC_METRIC_WINDOW_SECS,
        0,
    );
    let (format, dashboard) = options.source.into_dashboard_request(now_unix_secs);
    let request = IcMetricRequest::new(
        dashboard,
        IcMetricQuery::new(options.metric, start_unix_secs, end_unix_secs, options.step),
    );
    (format, request)
}

#[cfg(test)]
mod tests;
