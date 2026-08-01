use super::*;

#[test]
fn sns_canister_list_parses_lookup_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_canister_list_command,
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
        SnsLookupOptions::parse(
            [OsString::from("not-a-principal")],
            sns_canister_list_command,
        ),
        Err(SnsCommandError::Usage(_))
    ));
}
