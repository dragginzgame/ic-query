use super::*;
use crate::cli::clap::render_help;

#[test]
fn node_operator_list_parses_defaults_and_json_format() {
    let defaults = parse_test_options(
        leaf_list_command(
            &NODE_OPERATOR_SPEC,
            DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        ),
        &[],
        NnsLeafListOptions::from_matches,
    )
    .expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT
    );
    assert!(!defaults.verbose);

    let options = parse_test_options(
        leaf_list_command(
            &NODE_OPERATOR_SPEC,
            DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        ),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--verbose",
        ],
        NnsLeafListOptions::from_matches,
    )
    .expect("parse node-operator list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert!(options.verbose);
}

#[test]
fn node_operator_info_parses_input_and_json_format() {
    let options = parse_test_options(
        leaf_info_command(
            &NODE_OPERATOR_SPEC,
            DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        ),
        &["ryjl", "--json", "--source-endpoint", "https://icp-api.io"],
        NnsLeafInfoOptions::from_matches,
    )
    .expect("parse node-operator info");

    assert_eq!(options.input, "ryjl");
    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn node_operator_refresh_parses_defaults_and_export_options() {
    let defaults = parse_test_options(
        leaf_refresh_command(
            &NODE_OPERATOR_SPEC,
            DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        ),
        &[],
        NnsLeafRefreshOptions::from_matches,
    )
    .expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT
    );
    assert_eq!(
        defaults.lock_stale_after_seconds,
        DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(!defaults.dry_run);
    assert_eq!(defaults.output_path, None);

    let options = parse_test_options(
        leaf_refresh_command(
            &NODE_OPERATOR_SPEC,
            DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        ),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--lock-stale-after",
            "5m",
            "--dry-run",
            "--output",
            "operators.preview.json",
        ],
        NnsLeafRefreshOptions::from_matches,
    )
    .expect("parse node-operator refresh");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.lock_stale_after_seconds, 300);
    assert!(options.dry_run);
    assert_eq!(
        options.output_path,
        Some(PathBuf::from("operators.preview.json"))
    );
}

#[test]
fn node_operator_help_is_advertised_under_nns() {
    let nns = render_help(command());
    let node_operator = render_help(node_operator_command());
    let list = render_help(leaf_list_command(
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    ));
    let info = render_help(leaf_info_command(
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    ));
    let refresh = render_help(leaf_refresh_command(
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    ));

    assert!(nns.contains("node-operator"));
    assert!(node_operator.contains("List cached mainnet NNS node operators"));
    assert!(node_operator.contains("Show one cached mainnet NNS node operator"));
    assert!(node_operator.contains("Force-refresh and cache NNS node-operator metadata"));
    assert!(list.contains("icq nns node-operator list"));
    assert!(list.contains("--verbose"));
    assert!(list.contains("--json"));
    assert!(info.contains("icq nns node-operator info"));
    assert!(info.contains("node-operator|node-operator-prefix"));
    assert!(refresh.contains("icq nns node-operator refresh"));
    assert!(refresh.contains("--dry-run"));
}
