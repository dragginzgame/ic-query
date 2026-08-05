use super::*;

fn json_source_args() -> [&'static str; 3] {
    ["--json", "--source-endpoint", "https://icp-api.io"]
}

fn assert_topology_read_defaults(network: &str, format: OutputFormat, source_endpoint: &str) {
    assert_eq!(network, MAINNET_NETWORK);
    assert_eq!(format, OutputFormat::Text);
    assert_eq!(source_endpoint, DEFAULT_NNS_NODE_SOURCE_ENDPOINT);
}

fn assert_topology_json_source(format: OutputFormat, source_endpoint: &str) {
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(source_endpoint, "https://icp-api.io");
}

macro_rules! topology_read_options_parse_test {
    ($test_name:ident, $options:ident, $command:ident, $description:literal) => {
        #[test]
        fn $test_name() {
            let defaults = parse_test_options($command(), &[], $options::from_matches)
                .expect("parse defaults");
            assert_topology_read_defaults(
                &defaults.network,
                defaults.format,
                &defaults.source_endpoint,
            );

            let options =
                parse_test_options($command(), &json_source_args(), $options::from_matches)
                    .expect($description);
            assert_topology_json_source(options.format, &options.source_endpoint);
        }
    };
}

topology_read_options_parse_test!(
    topology_summary_parses_defaults_and_json_format,
    TopologySummaryOptions,
    topology_summary_command,
    "parse topology summary"
);
topology_read_options_parse_test!(
    topology_versions_parses_defaults_and_json_format,
    TopologyVersionsOptions,
    topology_versions_command,
    "parse topology versions"
);
topology_read_options_parse_test!(
    topology_coverage_parses_defaults_and_json_format,
    TopologyCoverageOptions,
    topology_coverage_command,
    "parse topology coverage"
);
topology_read_options_parse_test!(
    topology_check_parses_defaults_and_json_format,
    TopologyCheckOptions,
    topology_check_command,
    "parse topology check"
);
topology_read_options_parse_test!(
    topology_gaps_parses_defaults_and_json_format,
    TopologyGapsOptions,
    topology_gaps_command,
    "parse topology gaps"
);
topology_read_options_parse_test!(
    topology_capacity_parses_defaults_and_json_format,
    TopologyCapacityOptions,
    topology_capacity_command,
    "parse topology capacity"
);
topology_read_options_parse_test!(
    topology_regions_parses_defaults_and_json_format,
    TopologyRegionsOptions,
    topology_regions_command,
    "parse topology regions"
);
topology_read_options_parse_test!(
    topology_providers_parses_defaults_and_json_format,
    TopologyProvidersOptions,
    topology_providers_command,
    "parse topology providers"
);

#[test]
fn topology_refresh_parses_defaults_and_dry_run() {
    let defaults = parse_test_options(
        topology_refresh_command(),
        &[],
        TopologyRefreshOptions::from_matches,
    )
    .expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_NODE_SOURCE_ENDPOINT);
    assert_eq!(defaults.lock_stale_after_seconds, 30 * 60);
    assert!(!defaults.dry_run);

    let options = parse_test_options(
        topology_refresh_command(),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--lock-stale-after",
            "5m",
            "--dry-run",
        ],
        TopologyRefreshOptions::from_matches,
    )
    .expect("parse topology refresh");

    assert_topology_json_source(options.format, &options.source_endpoint);
    assert_eq!(options.lock_stale_after_seconds, 300);
    assert!(options.dry_run);
}
