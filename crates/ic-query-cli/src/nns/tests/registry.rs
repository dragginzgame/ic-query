use super::*;
use crate::cli::clap::render_help;

#[test]
fn registry_version_parses_defaults_and_json_format() {
    let defaults = parse_test_options(
        registry_version_command(),
        &[],
        RegistryVersionOptions::from_matches,
    )
    .expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT
    );

    let options = parse_test_options(
        registry_version_command(),
        &["--json", "--source-endpoint", "https://icp-api.io"],
        RegistryVersionOptions::from_matches,
    )
    .expect("parse registry version");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn registry_help_is_advertised_under_nns() {
    let nns = render_help(command());
    let registry = render_help(registry_command());
    let version = render_help(registry_version_command());

    assert!(nns.contains("registry"));
    assert!(registry.contains("Show the latest mainnet NNS registry version"));
    assert!(version.contains("icq nns registry version"));
    assert!(version.contains("Collection mode: Live query"));
    assert!(version.contains("--json"));
}
