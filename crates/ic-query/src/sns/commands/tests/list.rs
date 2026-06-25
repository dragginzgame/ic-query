use super::*;

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
