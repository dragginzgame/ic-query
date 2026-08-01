use super::*;

#[test]
fn sns_info_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_info_command,
    )
    .expect("parse info");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_metrics_parses_bounded_window_and_shared_lookup_options() {
    let options = SnsMetricsOptions::parse([
        OsString::from("1"),
        OsString::from("--window"),
        OsString::from("90d"),
        OsString::from("--json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
    ])
    .expect("parse metrics");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.time_window_seconds, 90 * 86_400);
}

#[test]
fn sns_metrics_uses_the_default_thirty_day_window() {
    let options = SnsMetricsOptions::parse([OsString::from("1")]).expect("parse metrics");

    assert_eq!(options.time_window_seconds, 30 * 86_400);
}

#[test]
fn sns_token_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_token_command,
    )
    .expect("parse token");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_params_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_params_command,
    )
    .expect("parse params");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_swap_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_swap_command,
    )
    .expect("parse swap");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_upgrade_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_upgrade_command,
    )
    .expect("parse upgrade");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}
