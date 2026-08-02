use super::*;

fn parse_proposals(args: &[&str]) -> Result<SnsProposalsOptions, SnsCommandError> {
    parse_fallible_test_options(
        sns_proposal_list_command(),
        args,
        SnsProposalsOptions::from_matches,
    )
}

#[test]
fn sns_proposals_parses_filters_and_json_format() {
    let options = parse_proposals(&[
        "1",
        "--json",
        "--source-endpoint",
        "https://icp-api.io",
        "--limit",
        "50",
        "--before",
        "100",
        "--status",
        "decided",
        "--topic",
        "governance",
        "--eligible",
        "yes",
        "--proposer",
        "00010203",
        "--query",
        "treasury",
        "--sort",
        "decided",
        "--asc",
        "--verbose",
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
    assert_eq!(options.eligibility, SnsProposalEligibilityArg::Yes);
    assert_eq!(options.proposer_neuron_id.as_deref(), Some("00010203"));
    assert_eq!(options.query.as_deref(), Some("treasury"));
    assert_eq!(options.sort, SnsProposalsSortArg::Decided);
    assert_eq!(options.sort_direction, SnsProposalSortDirection::Asc);
    assert!(options.verbose);
}

#[test]
fn sns_proposals_parses_local_sort_defaults_and_directions() {
    let title = parse_proposals(&["1", "--sort", "title"]).expect("parse title proposal sort");

    assert_eq!(title.sort, SnsProposalsSortArg::Title);
    assert_eq!(title.sort_direction, SnsProposalSortDirection::Asc);

    let action = parse_proposals(&["1", "--sort", "action"]).expect("parse action proposal sort");

    assert_eq!(action.sort, SnsProposalsSortArg::Action);
    assert_eq!(action.sort_direction, SnsProposalSortDirection::Asc);

    let status = parse_proposals(&["1", "--sort", "status"]).expect("parse status proposal sort");

    assert_eq!(status.sort, SnsProposalsSortArg::Status);
    assert_eq!(status.sort_direction, SnsProposalSortDirection::Asc);

    let topic = parse_proposals(&["1", "--sort", "topic"]).expect("parse topic proposal sort");

    assert_eq!(topic.sort, SnsProposalsSortArg::Topic);
    assert_eq!(topic.sort_direction, SnsProposalSortDirection::Asc);

    let proposer =
        parse_proposals(&["1", "--sort", "proposer"]).expect("parse proposer proposal sort");

    assert_eq!(proposer.sort, SnsProposalsSortArg::Proposer);
    assert_eq!(proposer.sort_direction, SnsProposalSortDirection::Asc);

    let title_desc = parse_proposals(&["1", "--sort", "title", "--desc"])
        .expect("parse descending title proposal sort");

    assert_eq!(title_desc.sort, SnsProposalsSortArg::Title);
    assert_eq!(title_desc.sort_direction, SnsProposalSortDirection::Desc);

    let total_votes =
        parse_proposals(&["1", "--sort", "total-votes"]).expect("parse total-votes proposal sort");

    assert_eq!(total_votes.sort, SnsProposalsSortArg::TotalVotes);
    assert_eq!(total_votes.sort_direction, SnsProposalSortDirection::Desc);

    let reject_cost =
        parse_proposals(&["1", "--sort", "reject-cost"]).expect("parse reject-cost proposal sort");

    assert_eq!(reject_cost.sort, SnsProposalsSortArg::RejectCost);
    assert_eq!(reject_cost.sort_direction, SnsProposalSortDirection::Desc);

    let reward_round = parse_proposals(&["1", "--sort", "reward-round"])
        .expect("parse reward-round proposal sort");

    assert_eq!(reward_round.sort, SnsProposalsSortArg::RewardRound);
    assert_eq!(reward_round.sort_direction, SnsProposalSortDirection::Desc);

    let ballots =
        parse_proposals(&["1", "--sort", "ballots", "--asc"]).expect("parse ballots proposal sort");

    assert_eq!(ballots.sort, SnsProposalsSortArg::Ballots);
    assert_eq!(ballots.sort_direction, SnsProposalSortDirection::Asc);
}

#[test]
fn sns_proposals_parses_extended_local_sort_values() {
    let action_id =
        parse_proposals(&["1", "--sort", "action-id"]).expect("parse action-id proposal sort");

    assert_eq!(action_id.sort, SnsProposalsSortArg::ActionId);
    assert_eq!(action_id.sort_direction, SnsProposalSortDirection::Desc);

    let tally_time =
        parse_proposals(&["1", "--sort", "tally-time"]).expect("parse tally-time proposal sort");

    assert_eq!(tally_time.sort, SnsProposalsSortArg::TallyTime);
    assert_eq!(tally_time.sort_direction, SnsProposalSortDirection::Desc);

    let eligible =
        parse_proposals(&["1", "--sort", "eligible"]).expect("parse eligible proposal sort");

    assert_eq!(eligible.sort, SnsProposalsSortArg::Eligible);
    assert_eq!(eligible.sort_direction, SnsProposalSortDirection::Desc);

    let reward_end =
        parse_proposals(&["1", "--sort", "reward-end"]).expect("parse reward-end proposal sort");

    assert_eq!(reward_end.sort, SnsProposalsSortArg::RewardEnd);
    assert_eq!(reward_end.sort_direction, SnsProposalSortDirection::Desc);
}

#[test]
fn sns_proposals_rejects_explicit_direction_for_api_sort() {
    let error = parse_proposals(&["1", "--sort", "api", "--desc"])
        .expect_err("api sort rejects explicit direction");

    assert!(matches!(error, SnsCommandError::Usage(_)));
}

#[test]
fn sns_proposal_parses_id_and_json_format() {
    let options = parse_test_options(
        sns_proposal_info_command(),
        &[
            "1",
            "42",
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--ballots",
            "--verbose",
        ],
        SnsProposalOptions::from_matches,
    )
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
    let options = parse_test_options(
        sns_proposal_refresh_command(),
        &[
            "1",
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--page-size",
            "50",
            "--max-pages",
            "3",
        ],
        SnsProposalsRefreshOptions::from_matches,
    )
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
    let list = parse_test_options(
        sns_proposal_cache_list_command(),
        &["--json"],
        SnsProposalsCacheListOptions::from_matches,
    )
    .expect("parse proposals cache list");
    assert_eq!(list.network, "ic");
    assert_eq!(list.format, OutputFormat::Json);

    let status = parse_test_options(
        sns_proposal_cache_status_command(),
        &["1", "--json"],
        SnsProposalsCacheStatusOptions::from_matches,
    )
    .expect("parse proposals cache status");
    assert_eq!(status.input, "1");
    assert_eq!(status.network, "ic");
    assert_eq!(status.format, OutputFormat::Json);
}
