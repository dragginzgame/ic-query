use super::*;
use crate::cli::{clap::render_help, common::OutputFormat};
use crate::ic::tests::parse_test_options;

#[test]
fn usage_discloses_live_dashboard_authority_and_bounds() {
    let network = render_help(command());
    let boundary_nodes = render_help(boundary_node_data_centers_command());
    let daily_stats = render_help(daily_stats_command());

    assert!(network.contains("Usage: network [COMMAND]"));
    assert!(network.contains("boundary-node-data-centers"));
    assert!(network.contains("daily-stats"));
    assert!(boundary_nodes.contains("Usage: boundary-node-data-centers [OPTIONS]"));
    assert!(boundary_nodes.contains("exactly one official Dashboard v4 request"));
    assert!(boundary_nodes.contains("locations that currently report zero nodes"));
    assert!(daily_stats.contains("Usage: daily-stats [OPTIONS]"));
    assert!(daily_stats.contains("exactly one official Dashboard v3 request"));
    assert!(daily_stats.contains("capped at one year and 366 returned rows"));
}

#[test]
fn boundary_node_data_center_options_preserve_format_and_endpoint() {
    let options = parse_test_options(
        boundary_node_data_centers_command(),
        &["--json", "--source-endpoint", "https://example.com/api/v4"],
        SourceOutputOptions::from_matches,
    )
    .expect("boundary-node options");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://example.com/api/v4");
}

#[test]
fn daily_stats_options_preserve_bounds_format_and_endpoint() {
    let (format, request) = parse_test_options(
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
        |matches| daily_stats_request(matches, 1_780_531_200),
    )
    .expect("daily-statistics options");

    assert_eq!(request.query.start_unix_secs, 1_784_937_600);
    assert_eq!(request.query.end_unix_secs, 1_785_542_400);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(
        request.dashboard.source_endpoint,
        "https://example.com/api/v3"
    );
}

#[test]
fn daily_stats_options_use_live_bounded_defaults() {
    let (format, request) = parse_test_options(daily_stats_command(), &[], |matches| {
        daily_stats_request(matches, 1_780_531_200)
    })
    .expect("default daily-statistics options");

    assert_eq!(
        request.query.start_unix_secs,
        1_780_531_200 - DEFAULT_IC_DAILY_STATS_WINDOW_SECS
    );
    assert_eq!(request.query.end_unix_secs, 1_780_531_200);
    assert_eq!(format, OutputFormat::Text);
    assert_eq!(
        request.dashboard.source_endpoint,
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT
    );
}
