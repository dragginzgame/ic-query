use super::*;
use crate::cli::clap::render_help;

#[test]
fn data_center_list_parses_defaults_and_json_format() {
    let defaults = parse_test_options(
        leaf_list_command(&DATA_CENTER_SPEC, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT),
        &[],
        NnsLeafListOptions::from_matches,
    )
    .expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT
    );
    assert!(!defaults.verbose);

    let options = parse_test_options(
        leaf_list_command(&DATA_CENTER_SPEC, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--verbose",
        ],
        NnsLeafListOptions::from_matches,
    )
    .expect("parse data-center list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert!(options.verbose);
}

#[test]
fn data_center_info_parses_input_and_json_format() {
    let options = parse_test_options(
        leaf_info_command(&DATA_CENTER_SPEC, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT),
        &["an1", "--json", "--source-endpoint", "https://icp-api.io"],
        NnsLeafInfoOptions::from_matches,
    )
    .expect("parse data-center info");

    assert_eq!(options.input, "an1");
    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn data_center_refresh_parses_defaults_and_export_options() {
    let defaults = parse_test_options(
        leaf_refresh_command(&DATA_CENTER_SPEC, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT),
        &[],
        NnsLeafRefreshOptions::from_matches,
    )
    .expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT
    );
    assert_eq!(
        defaults.lock_stale_after_seconds,
        DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(!defaults.dry_run);
    assert_eq!(defaults.output_path, None);

    let options = parse_test_options(
        leaf_refresh_command(&DATA_CENTER_SPEC, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--lock-stale-after",
            "5m",
            "--dry-run",
            "--output",
            "data-centers.preview.json",
        ],
        NnsLeafRefreshOptions::from_matches,
    )
    .expect("parse data-center refresh");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.lock_stale_after_seconds, 300);
    assert!(options.dry_run);
    assert_eq!(
        options.output_path,
        Some(PathBuf::from("data-centers.preview.json"))
    );
}

#[test]
fn data_center_help_is_advertised_under_nns() {
    let nns = render_help(command());
    let data_center = render_help(data_center_command());
    let list = render_help(leaf_list_command(
        &DATA_CENTER_SPEC,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    ));
    let info = render_help(leaf_info_command(
        &DATA_CENTER_SPEC,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    ));
    let refresh = render_help(leaf_refresh_command(
        &DATA_CENTER_SPEC,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    ));

    assert!(nns.contains("data-center"));
    assert!(data_center.contains("List cached mainnet NNS data centers"));
    assert!(data_center.contains("Show one cached mainnet NNS data center"));
    assert!(data_center.contains("Force-refresh and cache NNS data-center metadata"));
    assert!(list.contains("icq nns data-center list"));
    assert!(list.contains("--verbose"));
    assert!(list.contains("--json"));
    assert!(info.contains("icq nns data-center info"));
    assert!(info.contains("data-center|data-center-prefix"));
    assert!(refresh.contains("icq nns data-center refresh"));
    assert!(refresh.contains("--dry-run"));
}
