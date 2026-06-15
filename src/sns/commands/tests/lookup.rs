use super::*;

#[test]
fn sns_info_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--format"),
            OsString::from("json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_info_command,
        sns_info_usage,
    )
    .expect("parse info");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_token_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--format"),
            OsString::from("json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_token_command,
        sns_token_usage,
    )
    .expect("parse token");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}

#[test]
fn sns_params_parses_input_and_json_format() {
    let options = SnsLookupOptions::parse(
        [
            OsString::from("1"),
            OsString::from("--format"),
            OsString::from("json"),
            OsString::from("--source-endpoint"),
            OsString::from("https://icp-api.io"),
        ],
        sns_params_command,
        sns_params_usage,
    )
    .expect("parse params");

    assert_eq!(options.input, "1");
    assert_eq!(options.network, "ic");
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
}
