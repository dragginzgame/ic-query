use super::*;
use crate::cli::clap::render_help;
use ic_query::subnet_catalog::SubnetKind;

#[test]
fn node_list_parses_defaults_and_json_format() {
    let defaults = parse_test_options(node_list_command(), &[], node_list_options_from_matches)
        .expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_NODE_SOURCE_ENDPOINT);
    assert!(!defaults.verbose);

    let options = parse_test_options(
        node_list_command(),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--verbose",
            "--data-center",
            "zh2",
            "--node-provider",
            "7at4h",
            "--node-operator",
            "4lp6i",
            "--subnet",
            "tdb26",
            "--kind",
            "system",
        ],
        node_list_options_from_matches,
    )
    .expect("parse node list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert!(options.verbose);
    assert_eq!(options.filters.data_center.as_deref(), Some("zh2"));
    assert_eq!(options.filters.node_provider.as_deref(), Some("7at4h"));
    assert_eq!(options.filters.node_operator.as_deref(), Some("4lp6i"));
    assert_eq!(options.filters.subnet.as_deref(), Some("tdb26"));
    assert_eq!(options.filters.subnet_kind, Some(SubnetKind::System));
}

#[test]
fn node_info_parses_input_and_json_format() {
    let options = parse_test_options(
        leaf_info_command(&NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT),
        &["ryjl", "--json", "--source-endpoint", "https://icp-api.io"],
        NnsLeafInfoOptions::from_matches,
    )
    .expect("parse node info");

    assert_eq!(options.input, "ryjl");
    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn node_kind_filter_parses_every_registry_kind() {
    for (value, expected) in [
        ("application", SubnetKind::Application),
        ("cloud_engine", SubnetKind::CloudEngine),
        ("system", SubnetKind::System),
        ("unknown", SubnetKind::Unknown),
    ] {
        let options = parse_test_options(
            node_list_command(),
            &["--kind", value],
            node_list_options_from_matches,
        )
        .expect("parse supported Subnet kind");

        assert_eq!(options.filters.subnet_kind, Some(expected));
    }

    assert!(
        parse_test_options(
            node_list_command(),
            &["--kind", "public"],
            node_list_options_from_matches,
        )
        .is_err()
    );
}

#[test]
fn node_refresh_parses_defaults_and_export_options() {
    let defaults = parse_test_options(
        leaf_refresh_command(&NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT),
        &[],
        NnsLeafRefreshOptions::from_matches,
    )
    .expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_NODE_SOURCE_ENDPOINT);
    assert_eq!(
        defaults.lock_stale_after_seconds,
        DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(!defaults.dry_run);
    assert_eq!(defaults.output_path, None);

    let options = parse_test_options(
        leaf_refresh_command(&NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--lock-stale-after",
            "5m",
            "--dry-run",
            "--output",
            "nodes.preview.json",
        ],
        NnsLeafRefreshOptions::from_matches,
    )
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
    let nns = render_help(command());
    let node = render_help(node_command());
    let list = render_help(node_list_command());
    let info = render_help(leaf_info_command(
        &NODE_SPEC,
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
    ));
    let refresh = render_help(leaf_refresh_command(
        &NODE_SPEC,
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
    ));

    assert!(nns.contains("node"));
    assert!(node.contains("List cached mainnet NNS nodes"));
    assert!(node.contains("Show one cached mainnet NNS node"));
    assert!(node.contains("Force-refresh and cache NNS node metadata"));
    assert!(node.contains("Show observed operational status for IC nodes"));
    assert!(list.contains("icq nns node list"));
    assert!(list.contains("Collection mode: Cache-backed read"));
    assert!(list.contains("--verbose"));
    assert!(list.contains("--json"));
    assert!(list.contains("--data-center"));
    assert!(list.contains("--node-provider"));
    assert!(list.contains("--node-operator"));
    assert!(list.contains("--subnet"));
    assert!(list.contains("--kind"));
    assert!(info.contains("icq nns node info"));
    assert!(info.contains("node|node-prefix"));
    assert!(refresh.contains("icq nns node refresh"));
    assert!(refresh.contains("Collection mode: Forced live refresh"));
    assert!(refresh.contains("--dry-run"));
}
