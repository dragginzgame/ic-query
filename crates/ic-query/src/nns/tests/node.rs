use super::*;

#[test]
fn node_list_parses_defaults_and_json_format() {
    let defaults = node_list_options([]).expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_NODE_SOURCE_ENDPOINT);
    assert!(!defaults.verbose);

    let options = node_list_options([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--verbose"),
        OsString::from("--data-center"),
        OsString::from("zh2"),
        OsString::from("--node-provider"),
        OsString::from("7at4h"),
        OsString::from("--node-operator"),
        OsString::from("4lp6i"),
        OsString::from("--subnet"),
        OsString::from("tdb26"),
        OsString::from("--kind"),
        OsString::from("system"),
    ])
    .expect("parse node list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert!(options.verbose);
    assert_eq!(options.filters.data_center.as_deref(), Some("zh2"));
    assert_eq!(options.filters.node_provider.as_deref(), Some("7at4h"));
    assert_eq!(options.filters.node_operator.as_deref(), Some("4lp6i"));
    assert_eq!(options.filters.subnet.as_deref(), Some("tdb26"));
    assert_eq!(options.filters.subnet_kind.as_deref(), Some("system"));
}

#[test]
fn node_info_parses_input_and_json_format() {
    let options = node_info_options([
        OsString::from("ryjl"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
    ])
    .expect("parse node info");

    assert_eq!(options.input, "ryjl");
    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn node_refresh_parses_defaults_and_export_options() {
    let defaults = node_refresh_options([]).expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_NODE_SOURCE_ENDPOINT);
    assert_eq!(
        defaults.lock_stale_after_seconds,
        DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(!defaults.dry_run);
    assert_eq!(defaults.output_path, None);

    let options = node_refresh_options([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--lock-stale-after"),
        OsString::from("5m"),
        OsString::from("--dry-run"),
        OsString::from("--output"),
        OsString::from("nodes.preview.json"),
    ])
    .expect("parse node refresh");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.lock_stale_after_seconds, 300);
    assert!(options.dry_run);
    assert_eq!(
        options.output_path,
        Some(PathBuf::from("nodes.preview.json"))
    );
}

#[test]
fn node_help_is_advertised_under_nns() {
    let nns = usage();
    let node = node_usage();
    let list = node_list_usage();
    let info = node_info_usage();
    let refresh = node_refresh_usage();

    assert!(nns.contains("node"));
    assert!(node.contains("List cached mainnet NNS nodes"));
    assert!(node.contains("Show one cached mainnet NNS node"));
    assert!(node.contains("Force-refresh and cache NNS node metadata"));
    assert!(list.contains("icq nns node list"));
    assert!(list.contains("--verbose"));
    assert!(list.contains("--format json"));
    assert!(list.contains("--data-center"));
    assert!(list.contains("--node-provider"));
    assert!(list.contains("--node-operator"));
    assert!(list.contains("--subnet"));
    assert!(list.contains("--kind"));
    assert!(info.contains("icq nns node info"));
    assert!(info.contains("node|node-prefix"));
    assert!(refresh.contains("icq nns node refresh"));
    assert!(refresh.contains("--dry-run"));
}

#[test]
fn node_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("node"),
        OsString::from("list"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns node list"));
}
