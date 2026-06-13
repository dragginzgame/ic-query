use super::*;
use crate::test_support::assert_snapshot;

#[test]
fn sns_list_parses_defaults_and_json_format() {
    let defaults = SnsListOptions::parse([]).expect("parse defaults");
    assert_eq!(defaults.network, "ic");
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
    assert_eq!(defaults.sort, SnsListSortArg::Id);
    assert!(!defaults.verbose);

    let options = SnsListOptions::parse([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--sort"),
        OsString::from("name"),
        OsString::from("--verbose"),
    ])
    .expect("parse list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.sort, SnsListSortArg::Name);
    assert!(options.verbose);
}

#[test]
fn sns_info_parses_input_and_json_format() {
    let options = SnsInfoOptions::parse([
        OsString::from("1"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
    ])
    .expect("parse info");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_help_is_advertised() {
    let sns = usage();
    let list = sns_list_usage();
    let info = sns_info_usage();

    assert!(sns.contains("list"));
    assert!(sns.contains("info"));
    assert!(sns.contains("List deployed mainnet SNS instances"));
    assert!(sns.contains("Resolve a deployed SNS"));
    assert!(list.contains("icq sns list"));
    assert!(list.contains("--format json"));
    assert!(list.contains("--source-endpoint"));
    assert!(list.contains("--sort"));
    assert!(list.contains("--verbose"));
    assert!(info.contains("icq sns info"));
    assert!(info.contains("id|root-principal"));
}

#[test]
fn sns_list_usage_snapshot() {
    let expected = "\
List deployed mainnet SNS instances

Usage: icq sns list [OPTIONS]

Options:
      --format <text|json>     Output format; defaults to text [default: text] [possible values: text, json]
      --source-endpoint <url>  IC API endpoint used for SNS-W and governance metadata queries [default: https://icp-api.io]
      --verbose                Show full canister IDs in text output
      --sort <id|name>         Text/JSON row order; ids stay stable by root principal [default: id] [possible values: id, name]

Examples:
  icq sns list
  icq sns list --sort name
  icq sns list --verbose
  icq --network ic sns list --format json
  icq sns list --source-endpoint https://icp-api.io
";

    assert_snapshot("sns list usage", &sns_list_usage(), expected);
}
