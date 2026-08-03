use super::*;

#[test]
fn sns_info_parses_input_and_json_format() {
    let options = parse_test_options(
        sns_info_command(),
        &["1", "--json", "--source-endpoint", "https://icp-api.io"],
        SnsLookupOptions::from_matches,
    )
    .expect("parse info");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_metrics_parses_bounded_window_and_shared_lookup_options() {
    let options = parse_test_options(
        sns_metrics_command(),
        &[
            "1",
            "--window",
            "90d",
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
        ],
        SnsMetricsOptions::from_matches,
    )
    .expect("parse metrics");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.time_window_seconds, 90 * 86_400);
}

#[test]
fn sns_metrics_uses_the_default_thirty_day_window() {
    let options = parse_test_options(
        sns_metrics_command(),
        &["1"],
        SnsMetricsOptions::from_matches,
    )
    .expect("parse metrics");

    assert_eq!(options.time_window_seconds, 30 * 86_400);
}

#[test]
fn sns_token_parses_input_and_json_format() {
    let options = parse_test_options(
        sns_token_command(),
        &["1", "--json", "--source-endpoint", "https://icp-api.io"],
        SnsLookupOptions::from_matches,
    )
    .expect("parse token");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_parameters_parses_input_and_json_format() {
    let options = parse_test_options(
        sns_parameters_command(),
        &["1", "--json", "--source-endpoint", "https://icp-api.io"],
        SnsLookupOptions::from_matches,
    )
    .expect("parse parameters");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_swap_parses_input_and_json_format() {
    let options = parse_test_options(
        sns_swap_command(),
        &["1", "--json", "--source-endpoint", "https://icp-api.io"],
        SnsLookupOptions::from_matches,
    )
    .expect("parse swap");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_upgrade_parses_input_and_json_format() {
    let options = parse_test_options(
        sns_upgrade_command(),
        &["1", "--json", "--source-endpoint", "https://icp-api.io"],
        SnsLookupOptions::from_matches,
    )
    .expect("parse upgrade");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}
