use super::*;

fn json_source_args() -> Vec<OsString> {
    vec![
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
    ]
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
    ($test_name:ident, $options:ident, $description:literal) => {
        #[test]
        fn $test_name() {
            let defaults = $options::parse([]).expect("parse defaults");
            assert_topology_read_defaults(
                &defaults.network,
                defaults.format,
                &defaults.source_endpoint,
            );

            let options = $options::parse(json_source_args()).expect($description);
            assert_topology_json_source(options.format, &options.source_endpoint);
        }
    };
}

topology_read_options_parse_test!(
    topology_summary_parses_defaults_and_json_format,
    TopologySummaryOptions,
    "parse topology summary"
);
topology_read_options_parse_test!(
    topology_versions_parses_defaults_and_json_format,
    TopologyVersionsOptions,
    "parse topology versions"
);
topology_read_options_parse_test!(
    topology_coverage_parses_defaults_and_json_format,
    TopologyCoverageOptions,
    "parse topology coverage"
);
topology_read_options_parse_test!(
    topology_health_parses_defaults_and_json_format,
    TopologyHealthOptions,
    "parse topology health"
);
topology_read_options_parse_test!(
    topology_gaps_parses_defaults_and_json_format,
    TopologyGapsOptions,
    "parse topology gaps"
);
topology_read_options_parse_test!(
    topology_capacity_parses_defaults_and_json_format,
    TopologyCapacityOptions,
    "parse topology capacity"
);
topology_read_options_parse_test!(
    topology_regions_parses_defaults_and_json_format,
    TopologyRegionsOptions,
    "parse topology regions"
);
topology_read_options_parse_test!(
    topology_providers_parses_defaults_and_json_format,
    TopologyProvidersOptions,
    "parse topology providers"
);

#[test]
fn topology_refresh_parses_defaults_and_dry_run() {
    let defaults = TopologyRefreshOptions::parse([]).expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_NNS_NODE_SOURCE_ENDPOINT);
    assert_eq!(defaults.lock_stale_after_seconds, 30 * 60);
    assert!(!defaults.dry_run);

    let mut args = json_source_args();
    args.extend([
        OsString::from("--lock-stale-after"),
        OsString::from("5m"),
        OsString::from("--dry-run"),
    ]);
    let options = TopologyRefreshOptions::parse(args).expect("parse topology refresh");

    assert_topology_json_source(options.format, &options.source_endpoint);
    assert_eq!(options.lock_stale_after_seconds, 300);
    assert!(options.dry_run);
}
