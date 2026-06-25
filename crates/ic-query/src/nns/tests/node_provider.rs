use super::*;

#[test]
fn node_provider_list_parses_defaults_and_json_format() {
    let defaults = node_provider_list_options([]).expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_SOURCE_ENDPOINT);
    assert!(!defaults.verbose);

    let options = node_provider_list_options([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--verbose"),
    ])
    .expect("parse node-provider list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert!(options.verbose);
}

#[test]
fn node_provider_info_parses_input_and_json_format() {
    let options = node_provider_info_options([
        OsString::from("ryjl"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
    ])
    .expect("parse node-provider info");

    assert_eq!(options.input, "ryjl");
    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn node_provider_refresh_parses_defaults_and_export_options() {
    let defaults = node_provider_refresh_options([]).expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_SOURCE_ENDPOINT);
    assert_eq!(
        defaults.lock_stale_after_seconds,
        DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(!defaults.dry_run);
    assert_eq!(defaults.output_path, None);

    let options = node_provider_refresh_options([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--lock-stale-after"),
        OsString::from("5m"),
        OsString::from("--dry-run"),
        OsString::from("--output"),
        OsString::from("providers.preview.json"),
    ])
    .expect("parse node-provider refresh");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.lock_stale_after_seconds, 300);
    assert!(options.dry_run);
    assert_eq!(
        options.output_path,
        Some(PathBuf::from("providers.preview.json"))
    );
}

#[test]
fn node_provider_help_is_advertised_under_nns() {
    let nns = usage();
    let node_provider = node_provider_usage();
    let list = node_provider_list_usage();
    let info = node_provider_info_usage();
    let refresh = node_provider_refresh_usage();

    assert!(nns.contains("node-provider"));
    assert!(node_provider.contains("List cached mainnet NNS node providers"));
    assert!(node_provider.contains("Show one cached mainnet NNS node provider"));
    assert!(node_provider.contains("Force-refresh and cache NNS node-provider metadata"));
    assert!(list.contains("icq nns node-provider list"));
    assert!(list.contains("--verbose"));
    assert!(list.contains("--format json"));
    assert!(info.contains("icq nns node-provider info"));
    assert!(info.contains("node-provider|node-provider-prefix"));
    assert!(refresh.contains("icq nns node-provider refresh"));
    assert!(refresh.contains("--dry-run"));
}

#[test]
fn node_provider_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("node-provider"),
        OsString::from("list"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns node-provider list"));
}
