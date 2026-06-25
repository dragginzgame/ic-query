use super::*;

#[test]
fn data_center_list_parses_defaults_and_json_format() {
    let defaults = data_center_list_options([]).expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT
    );
    assert!(!defaults.verbose);

    let options = data_center_list_options([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--verbose"),
    ])
    .expect("parse data-center list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert!(options.verbose);
}

#[test]
fn data_center_info_parses_input_and_json_format() {
    let options = data_center_info_options([
        OsString::from("an1"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
    ])
    .expect("parse data-center info");

    assert_eq!(options.input, "an1");
    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn data_center_refresh_parses_defaults_and_export_options() {
    let defaults = data_center_refresh_options([]).expect("parse refresh defaults");

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

    let options = data_center_refresh_options([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--lock-stale-after"),
        OsString::from("5m"),
        OsString::from("--dry-run"),
        OsString::from("--output"),
        OsString::from("data-centers.preview.json"),
    ])
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
    let nns = usage();
    let data_center = data_center_usage();
    let list = data_center_list_usage();
    let info = data_center_info_usage();
    let refresh = data_center_refresh_usage();

    assert!(nns.contains("data-center"));
    assert!(data_center.contains("List cached mainnet NNS data centers"));
    assert!(data_center.contains("Show one cached mainnet NNS data center"));
    assert!(data_center.contains("Force-refresh and cache NNS data-center metadata"));
    assert!(list.contains("icq nns data-center list"));
    assert!(list.contains("--verbose"));
    assert!(list.contains("--format json"));
    assert!(info.contains("icq nns data-center info"));
    assert!(info.contains("data-center|data-center-prefix"));
    assert!(refresh.contains("icq nns data-center refresh"));
    assert!(refresh.contains("--dry-run"));
}

#[test]
fn data_center_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("data-center"),
        OsString::from("list"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns data-center list"));
}
