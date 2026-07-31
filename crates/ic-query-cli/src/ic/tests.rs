use super::*;

const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[test]
fn usage_discloses_live_dashboard_authority_and_command_shape() {
    let root = usage();
    let canister = canister_usage();
    let info = canister_info_usage();

    assert!(root.contains("Usage: icq ic [COMMAND]"));
    assert!(root.contains("canister"));
    assert!(canister.contains("Usage: icq ic canister [COMMAND]"));
    assert!(canister.contains("info"));
    assert!(info.contains("Usage: icq ic canister info [OPTIONS] <canister-id>"));
    assert!(info.contains("Live query; does not read or write a report cache."));
    assert!(info.contains("off-chain analytics authority"));
    assert!(info.contains("--source-endpoint"));
}

#[test]
fn canister_info_options_preserve_principal_format_and_endpoint() {
    let options = CanisterInfoOptions::parse([
        OsString::from(CANISTER_ID),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://example.com/api/v3"),
    ])
    .expect("canister options");

    assert_eq!(options.canister_id, CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://example.com/api/v3");
}

#[test]
fn canister_info_options_require_a_canister_id() {
    let error = CanisterInfoOptions::parse([]).expect_err("missing canister id");

    assert!(matches!(error, IcCommandError::Usage(message) if message.contains("required")));
}

#[test]
fn family_and_nested_help_return_without_network_calls() {
    for args in [
        &["help"][..],
        &["canister", "help"],
        &["canister", "info", "help"],
    ] {
        assert!(run(args.iter().map(OsString::from)).is_ok());
    }
}

#[test]
fn invalid_principal_fails_before_endpoint_or_network_use() {
    let error = run([
        OsString::from("canister"),
        OsString::from("info"),
        OsString::from("not a principal"),
        OsString::from("--source-endpoint"),
        OsString::from("not a URL"),
    ])
    .expect_err("invalid principal must fail");

    assert!(matches!(
        error,
        IcCommandError::Host(IcHostError::InvalidPrincipal {
            field: "canister_id",
            ..
        })
    ));
}
