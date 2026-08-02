use super::*;

#[test]
fn sns_canister_list_parses_lookup_and_json_format() {
    let options = parse_test_options(
        sns_canister_list_command(),
        &["1", "--json", "--source-endpoint", "https://icp-api.io"],
        SnsLookupOptions::from_matches,
    )
    .expect("parse canister list");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_canister_list_rejects_invalid_lookup() {
    assert!(matches!(
        parse_test_options(
            sns_canister_list_command(),
            &["not-a-principal"],
            SnsLookupOptions::from_matches,
        ),
        Err(SnsCommandError::Usage(_))
    ));
}
