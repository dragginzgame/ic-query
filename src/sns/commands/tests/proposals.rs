use super::*;

#[test]
fn sns_proposals_parses_filters_and_json_format() {
    let options = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--limit"),
        OsString::from("50"),
        OsString::from("--before"),
        OsString::from("100"),
        OsString::from("--status"),
        OsString::from("decided"),
        OsString::from("--topic"),
        OsString::from("any"),
        OsString::from("--sort"),
        OsString::from("decided"),
        OsString::from("--asc"),
        OsString::from("--verbose"),
    ])
    .expect("parse proposals");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.limit, 50);
    assert_eq!(options.before_proposal_id, Some(100));
    assert_eq!(options.status, SnsProposalStatusArg::Decided);
    assert_eq!(options.topic, SnsProposalTopicArg::Any);
    assert_eq!(options.sort, SnsProposalsSortArg::Decided);
    assert_eq!(options.sort_direction, SnsProposalSortDirection::Asc);
    assert!(options.verbose);
}

#[test]
fn sns_proposal_parses_id_and_json_format() {
    let options = SnsProposalOptions::parse([
        OsString::from("1"),
        OsString::from("42"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--ballots"),
        OsString::from("--verbose"),
    ])
    .expect("parse proposal");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.proposal_id, 42);
    assert!(options.show_ballots);
    assert!(options.verbose);
}

#[test]
fn sns_proposals_refresh_parses_page_controls() {
    let options = SnsProposalsRefreshOptions::parse([
        OsString::from("1"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--page-size"),
        OsString::from("50"),
        OsString::from("--max-pages"),
        OsString::from("3"),
    ])
    .expect("parse proposals refresh");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.page_size, 50);
    assert_eq!(options.max_pages, Some(3));
}

#[test]
fn sns_proposals_cache_parses_list_and_status_options() {
    let list =
        SnsProposalsCacheListOptions::parse([OsString::from("--format"), OsString::from("json")])
            .expect("parse proposals cache list");
    assert_eq!(list.network, "ic");
    assert_eq!(list.format, OutputFormat::Json);

    let status = SnsProposalsCacheStatusOptions::parse([
        OsString::from("1"),
        OsString::from("--format"),
        OsString::from("json"),
    ])
    .expect("parse proposals cache status");
    assert_eq!(status.input, "1");
    assert_eq!(status.network, "ic");
    assert_eq!(status.format, OutputFormat::Json);
}
