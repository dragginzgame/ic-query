use super::*;

#[test]
fn registry_version_parses_defaults_and_json_format() {
    let defaults = RegistryVersionOptions::parse([]).expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT
    );

    let options = RegistryVersionOptions::parse([
        OsString::from("--json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
    ])
    .expect("parse registry version");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn registry_help_is_advertised_under_nns() {
    let nns = usage();
    let registry = registry_usage();
    let version = registry_version_usage();

    assert!(nns.contains("registry"));
    assert!(registry.contains("Show the latest mainnet NNS registry version"));
    assert!(version.contains("icq nns registry version"));
    assert!(version.contains("Collection mode: Live query"));
    assert!(version.contains("--json"));
}
