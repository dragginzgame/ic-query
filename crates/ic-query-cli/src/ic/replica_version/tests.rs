use super::*;
use crate::cli::clap::render_help;
use crate::ic::tests::parse_test_options;
use ic_query::ic::DEFAULT_IC_REPLICA_VERSION_PAGE_LIMIT;

const VERSION_ID: &str = "e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3";

#[test]
fn usage_discloses_bounds_authority_and_runtime_limit() {
    let family = render_help(command());
    let info = render_help(info_command());
    let list = render_help(list_command());

    assert!(family.contains("Usage: replica-version [COMMAND]"));
    assert!(family.contains("info"));
    assert!(family.contains("list"));
    assert!(info.contains("exactly one official Dashboard detail request"));
    assert!(info.contains("prove which replica version"));
    assert!(list.contains("release rows"));
    assert!(list.contains("--max-proposal-index"));
    assert!(list.contains("not runtime-version evidence"));
}

#[test]
fn info_options_preserve_exact_target_format_and_endpoint() {
    let (format, request) = parse_test_options(
        info_command(),
        &[
            VERSION_ID,
            "--json",
            "--source-endpoint",
            "https://example.com/api/v3",
        ],
        |matches| info_request(matches, 1_780_531_200),
    )
    .expect("info options");

    assert_eq!(request.replica_version_id, VERSION_ID);
    assert_eq!(format, OutputFormat::Json);
    assert_eq!(
        request.dashboard.source_endpoint,
        "https://example.com/api/v3"
    );
}

#[test]
fn list_options_preserve_explicit_page_bounds() {
    let (format, request) = parse_test_options(
        list_command(),
        &[
            "--limit",
            "25",
            "--offset",
            "50",
            "--max-proposal-index",
            "438",
            "--json",
        ],
        |matches| list_request(matches, 1_780_531_200),
    )
    .expect("list options");

    assert_eq!(request.query.limit, 25);
    assert_eq!(request.query.offset, 50);
    assert_eq!(request.query.max_proposal_index, Some(438));
    assert_eq!(format, OutputFormat::Json);
}

#[test]
fn list_options_use_bounded_defaults() {
    let (format, request) = parse_test_options(list_command(), &[], |matches| {
        list_request(matches, 1_780_531_200)
    })
    .expect("default list options");

    assert_eq!(request.query.limit, DEFAULT_IC_REPLICA_VERSION_PAGE_LIMIT);
    assert_eq!(request.query.offset, 0);
    assert_eq!(request.query.max_proposal_index, None);
    assert_eq!(format, OutputFormat::Text);
}
