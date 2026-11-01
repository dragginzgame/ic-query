use super::*;

#[test]
fn sns_canister_list_parses_lookup_and_json_format() {
    let (format, options) = parse_test_options(
        sns_canister_list_command(),
        &["1", "--json", "--source-endpoint", "https://icp-api.io"],
        |m, n| lookup_request(m, n, 1_780_531_200),
    )
    .expect("parse canister list");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.now_unix_secs, 1_780_531_200);
}

#[test]
fn sns_canister_list_rejects_invalid_lookup() {
    assert!(matches!(
        parse_test_options(sns_canister_list_command(), &["not-a-principal"], |m, n| {
            lookup_request(m, n, 1_780_531_200)
        },),
        Err(SnsCommandError::Usage(_))
    ));
}
