use super::*;

const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[test]
fn usage_discloses_live_dashboard_authority_and_command_shape() {
    let root = usage();
    let canister = canister_usage();
    let info = canister_info_usage();
    let count = canister_count_usage();
    let page = canister_page_usage();

    assert!(root.contains("Usage: icq ic [COMMAND]"));
    assert!(root.contains("canister"));
    assert!(canister.contains("Usage: icq ic canister [COMMAND]"));
    assert!(canister.contains("info"));
    assert!(canister.contains("count"));
    assert!(canister.contains("page"));
    assert!(info.contains("Usage: icq ic canister info [OPTIONS] <canister-id>"));
    assert!(info.contains("Live query; does not read or write a report cache."));
    assert!(info.contains("off-chain analytics authority"));
    assert!(info.contains("--source-endpoint"));
    assert!(count.contains("Usage: icq ic canister count [OPTIONS]"));
    assert!(count.contains("exactly one official Dashboard count request"));
    assert!(page.contains("Usage: icq ic canister page [OPTIONS]"));
    assert!(page.contains("limit is capped at 100"));
    assert!(page.contains("No cache is used"));
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
fn canister_count_options_preserve_official_filters() {
    let options = CanisterCountOptions::parse([
        OsString::from("--has-name"),
        OsString::from("true"),
        OsString::from("--subnet-id"),
        OsString::from("tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe"),
        OsString::from("--language"),
        OsString::from("rust"),
        OsString::from("--language"),
        OsString::from("motoko"),
        OsString::from("--canister-type"),
        OsString::from("ledger"),
        OsString::from("--query"),
        OsString::from("ICP Ledger"),
    ])
    .expect("count options");

    assert_eq!(options.filters.has_name, Some(true));
    assert_eq!(options.filters.languages, ["rust", "motoko"]);
    assert_eq!(options.filters.canister_types, ["ledger"]);
    assert_eq!(options.filters.query.as_deref(), Some("ICP Ledger"));
    assert_eq!(
        options.source_endpoint,
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT
    );
}

#[test]
fn canister_page_options_are_bounded_and_cursors_are_exclusive() {
    let options = CanisterPageOptions::parse([
        OsString::from("--limit"),
        OsString::from("100"),
        OsString::from("--after"),
        OsString::from(CANISTER_ID),
    ])
    .expect("page options");

    assert_eq!(options.limit, MAX_IC_CANISTER_PAGE_LIMIT);
    assert_eq!(options.after.as_deref(), Some(CANISTER_ID));
    assert_eq!(options.before, None);

    let excessive = CanisterPageOptions::parse([OsString::from("--limit"), OsString::from("101")])
        .expect_err("page limit above API maximum must fail");
    assert!(matches!(excessive, IcCommandError::Usage(_)));

    let conflicting = CanisterPageOptions::parse([
        OsString::from("--after"),
        OsString::from(CANISTER_ID),
        OsString::from("--before"),
        OsString::from(CANISTER_ID),
    ])
    .expect_err("page cursors must be exclusive");
    assert!(matches!(conflicting, IcCommandError::Usage(_)));
}

#[test]
fn cli_and_library_page_defaults_remain_aligned() {
    let options = CanisterPageOptions::parse([]).expect("default page options");

    assert_eq!(options.limit, ic_query::ic::DEFAULT_IC_CANISTER_PAGE_LIMIT);
}

#[test]
fn family_and_nested_help_return_without_network_calls() {
    for args in [
        &["help"][..],
        &["canister", "help"],
        &["canister", "info", "help"],
        &["canister", "count", "help"],
        &["canister", "page", "help"],
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
