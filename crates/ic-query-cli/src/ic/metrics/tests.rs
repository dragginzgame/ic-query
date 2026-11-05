use super::*;
use crate::cli::clap::render_help;
use crate::ic::tests::parse_test_options;

#[test]
fn usage_discloses_live_dashboard_authority_and_bounds() {
    let usage = render_help(command());

    assert!(usage.contains("Usage: metrics [OPTIONS] <metric>"));
    assert!(usage.contains("exactly one official Dashboard Metrics API request"));
    assert!(usage.contains("capped at 1000 observations"));
    assert!(usage.contains("off-chain, non-certified API"));
}

#[test]
fn options_preserve_official_kind_bounds_and_endpoint() {
    let (format, request) = parse_test_options(
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
        |matches| metric_request(matches, 1_780_531_200),
    )
    .expect("metric options");

    assert_eq!(request.query.metric, IcMetricKind::CycleBurnRate);
    assert_eq!(request.query.start_unix_secs, 1_700_000_000);
    assert_eq!(request.query.end_unix_secs, 1_700_003_600);
    assert_eq!(request.query.step_secs, 600);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(
        request.dashboard.source_endpoint,
        "https://example.com/api/v1"
    );
}

#[test]
fn options_use_bounded_defaults_and_reject_unknown_kinds() {
    let (_, request) = parse_test_options(command(), &["instruction-rate"], |matches| {
        metric_request(matches, 1_780_531_200)
    })
    .expect("default options");

    assert_eq!(request.query.step_secs, DEFAULT_IC_METRIC_STEP_SECS);
    assert_eq!(
        request.dashboard.source_endpoint,
        DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT
    );
    assert_eq!(
        request.query.start_unix_secs,
        1_780_531_200 - DEFAULT_IC_METRIC_WINDOW_SECS
    );
    assert_eq!(request.query.end_unix_secs, 1_780_531_200);

    let error = parse_test_options(command(), &["made-up-rate"], |matches| {
        metric_request(matches, 1_780_531_200)
    })
    .expect_err("unknown metric must fail");
    assert!(matches!(error, IcCommandError::Usage(_)));
}
