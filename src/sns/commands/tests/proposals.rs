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
        OsString::from("governance"),
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
    assert_eq!(options.topic, SnsProposalTopicArg::Governance);
    assert_eq!(options.sort, SnsProposalsSortArg::Decided);
    assert_eq!(options.sort_direction, SnsProposalSortDirection::Asc);
    assert!(options.verbose);
}

#[test]
fn sns_proposals_parses_local_sort_defaults_and_directions() {
    let title = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("title"),
    ])
    .expect("parse title proposal sort");

    assert_eq!(title.sort, SnsProposalsSortArg::Title);
    assert_eq!(title.sort_direction, SnsProposalSortDirection::Asc);

    let action = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("action"),
    ])
    .expect("parse action proposal sort");

    assert_eq!(action.sort, SnsProposalsSortArg::Action);
    assert_eq!(action.sort_direction, SnsProposalSortDirection::Asc);

    let status = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("status"),
    ])
    .expect("parse status proposal sort");

    assert_eq!(status.sort, SnsProposalsSortArg::Status);
    assert_eq!(status.sort_direction, SnsProposalSortDirection::Asc);

    let topic = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("topic"),
    ])
    .expect("parse topic proposal sort");

    assert_eq!(topic.sort, SnsProposalsSortArg::Topic);
    assert_eq!(topic.sort_direction, SnsProposalSortDirection::Asc);

    let proposer = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("proposer"),
    ])
    .expect("parse proposer proposal sort");

    assert_eq!(proposer.sort, SnsProposalsSortArg::Proposer);
    assert_eq!(proposer.sort_direction, SnsProposalSortDirection::Asc);

    let title_desc = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("title"),
        OsString::from("--desc"),
    ])
    .expect("parse descending title proposal sort");

    assert_eq!(title_desc.sort, SnsProposalsSortArg::Title);
    assert_eq!(title_desc.sort_direction, SnsProposalSortDirection::Desc);

    let total_votes = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("total-votes"),
    ])
    .expect("parse total-votes proposal sort");

    assert_eq!(total_votes.sort, SnsProposalsSortArg::TotalVotes);
    assert_eq!(total_votes.sort_direction, SnsProposalSortDirection::Desc);

    let reject_cost = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("reject-cost"),
    ])
    .expect("parse reject-cost proposal sort");

    assert_eq!(reject_cost.sort, SnsProposalsSortArg::RejectCost);
    assert_eq!(reject_cost.sort_direction, SnsProposalSortDirection::Desc);

    let reward_round = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("reward-round"),
    ])
    .expect("parse reward-round proposal sort");

    assert_eq!(reward_round.sort, SnsProposalsSortArg::RewardRound);
    assert_eq!(reward_round.sort_direction, SnsProposalSortDirection::Desc);

    let ballots = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("ballots"),
        OsString::from("--asc"),
    ])
    .expect("parse ballots proposal sort");

    assert_eq!(ballots.sort, SnsProposalsSortArg::Ballots);
    assert_eq!(ballots.sort_direction, SnsProposalSortDirection::Asc);
}

#[test]
fn sns_proposals_rejects_explicit_direction_for_api_sort() {
    let error = SnsProposalsOptions::parse([
        OsString::from("1"),
        OsString::from("--sort"),
        OsString::from("api"),
        OsString::from("--desc"),
    ])
    .expect_err("api sort rejects explicit direction");

    assert!(matches!(error, SnsCommandError::Usage(_)));
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
