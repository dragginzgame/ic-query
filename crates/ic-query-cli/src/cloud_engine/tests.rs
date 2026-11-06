use super::*;
use crate::cli::common::SOURCE_ENDPOINT_ARG;
use crate::cli::{
    clap::{parse_matches, render_help},
    common::OutputFormat,
};
use std::ffi::OsString;

#[test]
fn usage_discloses_bounded_uncertified_reports() {
    let usage = render_help(command());
    assert!(usage.contains("Usage: cloud-engine [COMMAND]"));
    assert!(usage.contains("info"));
    assert!(usage.contains("list"));
    assert!(usage.contains("node"));
    assert!(usage.contains("prices"));
    assert!(usage.contains("provider"));

    let info = render_help(info_command());
    assert!(info.contains("<subnet-id>"));
    assert!(info.contains("operator queries"));
    assert!(info.contains("certified"));

    let prices = render_help(prices_command());
    assert!(prices.contains("exactly two control-plane queries"));
    assert!(prices.contains("1,000"));
    assert!(prices.contains(COLLECTION_MODE_LIVE));

    let list = render_help(list_command());
    assert!(list.contains("one exact public control-plane"));
    assert!(list.contains("--registry-source-endpoint"));
    assert!(list.contains("up to 100"));
    assert!(list.contains(LIST_COLLECTION_MODE));
}

#[tokio::test(flavor = "current_thread")]
async fn help_and_version_do_not_make_live_calls() {
    for args in [
        &["cloud-engine", "--help"][..],
        &["cloud-engine", "info", "--help"],
        &["cloud-engine", "list", "--help"],
        &["cloud-engine", "node", "--help"],
        &["cloud-engine", "node", "info", "--help"],
        &["cloud-engine", "node", "list", "--help"],
        &["cloud-engine", "prices", "--help"],
        &["cloud-engine", "provider", "--help"],
        &["cloud-engine", "provider", "info", "--help"],
        &["cloud-engine", "provider", "list", "--help"],
        &["cloud-engine", "--version"],
    ] {
        assert!(crate::run(args.iter().map(OsString::from)).await.is_ok());
    }
}

#[test]
fn report_options_default_to_native_endpoint_and_text() {
    let matches = parse_matches(prices_command(), Vec::<OsString>::new())
        .expect("parse default CloudEngine options");

    assert_eq!(
        matches
            .get_one::<String>(SOURCE_ENDPOINT_ARG)
            .expect("validated source endpoint"),
        DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT
    );
    assert_eq!(output_format(&matches), OutputFormat::Text);
}

#[test]
fn list_options_keep_registry_and_control_plane_endpoints_explicit() {
    let matches = parse_matches(list_command(), Vec::<OsString>::new())
        .expect("parse default CloudEngine list options");

    assert_eq!(
        matches
            .get_one::<String>(REGISTRY_SOURCE_ENDPOINT_ARG)
            .expect("validated source endpoint"),
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT
    );
    assert_eq!(
        matches
            .get_one::<String>(SOURCE_ENDPOINT_ARG)
            .expect("validated source endpoint"),
        DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT
    );
    assert_eq!(output_format(&matches), OutputFormat::Text);
}
