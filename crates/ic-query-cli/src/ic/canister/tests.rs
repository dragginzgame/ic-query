use super::*;
use crate::cli::clap::render_help;
use crate::ic::tests::parse_test_options;
use ic_query::ic::IcHostError;
use std::ffi::OsString;

const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[test]
fn usage_discloses_live_dashboard_authority_and_command_shape() {
    let canister = render_help(command());
    let info = render_help(info_command());
    let count = render_help(count_command());
    let page = render_help(page_command());

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
    let (format, request) = parse_test_options(
        info_command(),
        &[
            CANISTER_ID,
            "--json",
            "--source-endpoint",
            "https://example.com/api/v3",
        ],
        |matches| info_request(matches, 1_780_531_200),
    )
    .expect("canister options");

    assert_eq!(request.canister_id, CANISTER_ID);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(
        request.dashboard.source_endpoint,
        "https://example.com/api/v3"
    );
}

#[test]
fn canister_info_options_require_a_canister_id() {
    let error = parse_test_options(info_command(), &[], |matches| {
        info_request(matches, 1_780_531_200)
    })
    .expect_err("missing canister id");

    assert!(matches!(error, IcCommandError::Usage(message) if message.contains("required")));
}

#[test]
fn canister_count_options_preserve_official_filters() {
    let (_, request) = parse_test_options(
        count_command(),
        &[
            "--has-name",
            "true",
            "--subnet-id",
            "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe",
            "--language",
            "rust",
            "--language",
            "motoko",
            "--canister-type",
            "ledger",
            "--query",
            "ICP Ledger",
        ],
        |matches| count_request(matches, 1_780_531_200),
    )
    .expect("count options");

    assert_eq!(request.filters.has_name, Some(true));
    assert_eq!(request.filters.languages, ["rust", "motoko"]);
    assert_eq!(request.filters.canister_types, ["ledger"]);
    assert_eq!(request.filters.query.as_deref(), Some("ICP Ledger"));
    assert_eq!(
        request.dashboard.source_endpoint,
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT
    );
}

#[test]
fn canister_page_options_are_bounded_and_cursors_are_exclusive() {
    let (_, request) = parse_test_options(
        page_command(),
        &["--limit", "100", "--after", CANISTER_ID],
        |matches| page_request(matches, 1_780_531_200),
    )
    .expect("page options");

    assert_eq!(request.limit, MAX_IC_CANISTER_PAGE_LIMIT);
    assert_eq!(request.after.as_deref(), Some(CANISTER_ID));
    assert_eq!(request.before, None);

    let excessive = parse_test_options(page_command(), &["--limit", "101"], |matches| {
        page_request(matches, 1_780_531_200)
    })
    .expect_err("page limit above API maximum must fail");
    assert!(matches!(excessive, IcCommandError::Usage(_)));

    let conflicting = parse_test_options(
        page_command(),
        &["--after", CANISTER_ID, "--before", CANISTER_ID],
        |matches| page_request(matches, 1_780_531_200),
    )
    .expect_err("page cursors must be exclusive");
    assert!(matches!(conflicting, IcCommandError::Usage(_)));
}

#[test]
fn cli_and_library_page_defaults_remain_aligned() {
    let (_, request) = parse_test_options(page_command(), &[], |matches| {
        page_request(matches, 1_780_531_200)
    })
    .expect("default page options");

    assert_eq!(request.limit, ic_query::ic::DEFAULT_IC_CANISTER_PAGE_LIMIT);
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_principal_fails_before_endpoint_or_network_use() {
    let error = crate::run([
        OsString::from("ic"),
        OsString::from("canister"),
        OsString::from("info"),
        OsString::from("not a principal"),
        OsString::from("--source-endpoint"),
        OsString::from("not a URL"),
    ])
    .await
    .expect_err("invalid principal must fail");

    assert!(matches!(
        error,
        crate::IcqCliError::Ic(IcCommandError::Host(IcHostError::InvalidPrincipal {
            field: "canister_id",
            ..
        }))
    ));
}
