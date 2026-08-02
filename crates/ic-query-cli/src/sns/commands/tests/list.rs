use super::*;

#[test]
fn sns_list_parses_defaults_and_json_format() {
    let defaults = parse_test_options(sns_list_command(), &[], SnsListOptions::from_matches)
        .expect("parse defaults");
    assert_eq!(defaults.network, "ic");
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(defaults.source_endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
    assert_eq!(defaults.sort, SnsListSortArg::Id);
    assert!(!defaults.verbose);

    let options = parse_test_options(
        sns_list_command(),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--sort",
            "name",
            "--verbose",
        ],
        SnsListOptions::from_matches,
    )
    .expect("parse list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.sort, SnsListSortArg::Name);
    assert!(options.verbose);
}
