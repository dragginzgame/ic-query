use super::*;
use crate::cli::clap::render_help;

#[test]
fn node_provider_list_parses_defaults_and_json_format() {
    let defaults = parse_test_options(
        leaf_list_command(
            &NODE_PROVIDER_SPEC,
            DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        ),
        &[],
        NnsLeafListOptions::from_matches,
    )
    .expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT
    );
    assert!(!defaults.verbose);

    let options = parse_test_options(
        leaf_list_command(
            &NODE_PROVIDER_SPEC,
            DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        ),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--verbose",
        ],
        NnsLeafListOptions::from_matches,
    )
    .expect("parse node-provider list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert!(options.verbose);
}

#[test]
fn node_provider_info_parses_input_and_json_format() {
    let options = parse_test_options(
        leaf_info_command(
            &NODE_PROVIDER_SPEC,
            DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        ),
        &["ryjl", "--json", "--source-endpoint", "https://icp-api.io"],
        NnsLeafInfoOptions::from_matches,
    )
    .expect("parse node-provider info");

    assert_eq!(options.input, "ryjl");
    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn node_provider_refresh_parses_defaults_and_export_options() {
    let defaults = parse_test_options(
        leaf_refresh_command(
            &NODE_PROVIDER_SPEC,
            DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        ),
        &[],
        NnsLeafRefreshOptions::from_matches,
    )
    .expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT
    );
    assert_eq!(
        defaults.lock_stale_after_seconds,
        DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(!defaults.dry_run);
    assert_eq!(defaults.output_path, None);

    let options = parse_test_options(
        leaf_refresh_command(
            &NODE_PROVIDER_SPEC,
            DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        ),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--lock-stale-after",
            "5m",
            "--dry-run",
            "--output",
            "providers.preview.json",
        ],
        NnsLeafRefreshOptions::from_matches,
    )
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
    let nns = render_help(command());
    let node_provider = render_help(node_provider_command());
    let list = render_help(leaf_list_command(
        &NODE_PROVIDER_SPEC,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
    ));
    let info = render_help(leaf_info_command(
        &NODE_PROVIDER_SPEC,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
    ));
    let refresh = render_help(leaf_refresh_command(
        &NODE_PROVIDER_SPEC,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
    ));

    assert!(nns.contains("node-provider"));
    assert!(node_provider.contains("List cached mainnet NNS node providers"));
    assert!(node_provider.contains("Show one cached mainnet NNS node provider"));
    assert!(node_provider.contains("Force-refresh and cache NNS node-provider metadata"));
    assert!(node_provider.contains("Show observed operational status grouped by node provider"));
    assert!(list.contains("icq nns node-provider list"));
    assert!(list.contains("--verbose"));
    assert!(list.contains("--json"));
    assert!(info.contains("icq nns node-provider info"));
    assert!(info.contains("node-provider|node-provider-prefix"));
    assert!(refresh.contains("icq nns node-provider refresh"));
    assert!(refresh.contains("--dry-run"));
}
