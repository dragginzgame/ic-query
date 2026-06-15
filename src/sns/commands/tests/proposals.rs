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
        OsString::from("open"),
        OsString::from("--verbose"),
    ])
    .expect("parse proposals");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.limit, 50);
    assert_eq!(options.before_proposal_id, Some(100));
    assert_eq!(options.status, SnsProposalStatusArg::Open);
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
        OsString::from("--verbose"),
    ])
    .expect("parse proposal");

    assert_eq!(options.lookup.input, "1");
    assert_eq!(options.lookup.network, "ic");
    assert_eq!(options.lookup.format, OutputFormat::Json);
    assert_eq!(options.lookup.source_endpoint, "https://icp-api.io");
    assert_eq!(options.proposal_id, 42);
    assert!(options.verbose);
}
