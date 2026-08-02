//! Module: ic::metrics
//!
//! Responsibility: parse and run bounded official Dashboard metric queries.
//! Does not own: Dashboard transport, report construction, or text rendering.
//! Boundary: exposes the metric command to the IC CLI facade.

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
    DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT, DEFAULT_IC_METRIC_STEP_SECS,
    DEFAULT_IC_METRIC_WINDOW_SECS, IcMetricKind, IcMetricQuery, IcMetricRequest,
    MAX_IC_METRIC_STEP_SECS, MIN_IC_METRIC_TIMESTAMP, build_ic_metric_report,
    ic_metric_report_text,
};

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

pub(super) fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
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

pub(super) fn command() -> ClapCommand {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;

    #[test]
    fn usage_discloses_live_dashboard_authority_and_bounds() {
        let usage = render_help(command());

        assert!(usage.contains("Usage: icq ic metrics [OPTIONS] <metric>"));
        assert!(usage.contains("exactly one official Dashboard Metrics API request"));
        assert!(usage.contains("capped at 1000 observations"));
        assert!(usage.contains("off-chain, non-certified API"));
    }

    #[test]
    fn options_preserve_official_kind_bounds_and_endpoint() {
        let options = parse_test_options(
            command(),
            &[
                "cycle-burn-rate",
                "--start",
                "1700000000",
                "--end",
                "1700003600",
                "--step",
                "600",
                "--json",
                "--source-endpoint",
                "https://example.com/api/v1",
            ],
            MetricOptions::from_matches,
        )
        .expect("metric options");

        assert_eq!(options.metric, IcMetricKind::CycleBurnRate);
        assert_eq!(options.start_unix_secs, Some(1_700_000_000));
        assert_eq!(options.end_unix_secs, Some(1_700_003_600));
        assert_eq!(options.step_secs, 600);
        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.source_endpoint, "https://example.com/api/v1");
    }

    #[test]
    fn options_use_bounded_defaults_and_reject_unknown_kinds() {
        let options = parse_test_options(
            command(),
            &["instruction-rate"],
            MetricOptions::from_matches,
        )
        .expect("default options");

        assert_eq!(options.step_secs, DEFAULT_IC_METRIC_STEP_SECS);
        assert_eq!(
            options.source_endpoint,
            DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT
        );
        assert_eq!(options.start_unix_secs, None);
        assert_eq!(options.end_unix_secs, None);

        let error = parse_test_options(command(), &["made-up-rate"], MetricOptions::from_matches)
            .expect_err("unknown metric must fail");
        assert!(matches!(error, IcCommandError::Usage(_)));
    }
}
