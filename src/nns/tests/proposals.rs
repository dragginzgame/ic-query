use super::*;

#[test]
fn nns_proposal_list_parses_defaults_and_json_format() {
    let defaults = NnsProposalListOptions::parse_list([]).expect("parse defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT
    );
    assert_eq!(defaults.limit, 25);
    assert_eq!(defaults.before_proposal_id, None);
    assert_eq!(defaults.status, NnsProposalStatusFilter::Any);
    assert_eq!(defaults.reward_status, NnsProposalRewardStatusFilter::Any);
    assert_eq!(defaults.topic, NnsProposalTopicFilter::Any);
    assert_eq!(defaults.sort, NnsProposalListSort::Api);
    assert_eq!(defaults.sort_direction, NnsProposalSortDirection::Desc);
    assert_eq!(defaults.status.as_str(), NNS_PROPOSAL_STATUS_ANY_LABEL);
    assert_eq!(
        defaults.reward_status.as_str(),
        NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL
    );
    assert_eq!(defaults.topic.as_str(), NNS_PROPOSAL_TOPIC_ANY_LABEL);
    assert_eq!(defaults.sort.as_str(), NNS_PROPOSAL_SORT_API_LABEL);
    assert_eq!(
        defaults.sort.direction_label(defaults.sort_direction),
        NNS_PROPOSAL_SORT_NONE_LABEL
    );
    assert!(!defaults.verbose);

    let options = NnsProposalListOptions::parse_list([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--limit"),
        OsString::from("50"),
        OsString::from("--before"),
        OsString::from("132000"),
        OsString::from("--status"),
        OsString::from(NNS_PROPOSAL_STATUS_EXECUTED_LABEL),
        OsString::from("--reward-status"),
        OsString::from(NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL),
        OsString::from("--topic"),
        OsString::from(NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL),
        OsString::from("--sort"),
        OsString::from(NNS_PROPOSAL_SORT_TITLE_LABEL),
        OsString::from("--asc"),
        OsString::from("--verbose"),
    ])
    .expect("parse nns proposal list");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.limit, 50);
    assert_eq!(options.before_proposal_id, Some(132_000));
    assert_eq!(options.status, NnsProposalStatusFilter::Executed);
    assert_eq!(
        options.reward_status,
        NnsProposalRewardStatusFilter::Settled
    );
    assert_eq!(options.topic, NnsProposalTopicFilter::Governance);
    assert_eq!(options.sort, NnsProposalListSort::Title);
    assert_eq!(options.sort_direction, NnsProposalSortDirection::Asc);
    assert_eq!(options.status.as_str(), NNS_PROPOSAL_STATUS_EXECUTED_LABEL);
    assert_eq!(
        options.reward_status.as_str(),
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL
    );
    assert_eq!(options.topic.as_str(), NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL);
    assert_eq!(options.sort.as_str(), NNS_PROPOSAL_SORT_TITLE_LABEL);
    assert_eq!(
        options.sort.direction_label(options.sort_direction),
        NNS_PROPOSAL_SORT_ASC_LABEL
    );
    assert!(options.verbose);

    let grouped_options = NnsProposalListOptions::parse_list([
        OsString::from("--limit"),
        OsString::from("10"),
        OsString::from("--reward-status"),
        OsString::from(NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL),
    ])
    .expect("parse nns proposal list");

    assert_eq!(grouped_options.limit, 10);
    assert_eq!(
        grouped_options.reward_status,
        NnsProposalRewardStatusFilter::Settled
    );
}

#[test]
fn nns_proposal_parses_id_and_json_format() {
    let options = NnsProposalOptions::parse_info([
        OsString::from("132411"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--ballots"),
        OsString::from("--verbose"),
    ])
    .expect("parse nns proposal info");

    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.proposal_id, 132_411);
    assert!(options.show_ballots);
    assert!(options.verbose);

    let grouped_options = NnsProposalOptions::parse_info([
        OsString::from("132411"),
        OsString::from("--ballots"),
        OsString::from("--verbose"),
    ])
    .expect("parse nns proposal info");

    assert_eq!(grouped_options.proposal_id, 132_411);
    assert!(grouped_options.show_ballots);
    assert!(grouped_options.verbose);
}

#[test]
fn nns_proposal_refresh_parses_cache_options() {
    let defaults = NnsProposalRefreshOptions::parse([]).expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT
    );
    assert_eq!(defaults.page_size, 100);
    assert_eq!(defaults.max_pages, None);

    let options = NnsProposalRefreshOptions::parse([
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--page-size"),
        OsString::from("25"),
        OsString::from("--max-pages"),
        OsString::from("2"),
    ])
    .expect("parse refresh options");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.page_size, 25);
    assert_eq!(options.max_pages, Some(2));
}

#[test]
fn nns_proposal_cache_options_parse_json_format() {
    let list =
        NnsProposalCacheListOptions::parse([OsString::from("--format"), OsString::from("json")])
            .expect("parse cache list");
    let status =
        NnsProposalCacheStatusOptions::parse([OsString::from("--format"), OsString::from("json")])
            .expect("parse cache status");

    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.format, OutputFormat::Json);
    assert_eq!(status.network, MAINNET_NETWORK);
    assert_eq!(status.format, OutputFormat::Json);
}

#[test]
fn nns_proposal_help_is_advertised_under_nns() {
    let nns = usage();
    let proposal = nns_proposal_usage();
    let proposal_list = nns_proposal_list_usage();
    let proposal_info = nns_proposal_info_usage();
    let proposal_refresh = nns_proposal_refresh_usage();
    let proposal_cache = nns_proposal_cache_usage();
    let proposal_cache_list = nns_proposal_cache_list_usage();
    let proposal_cache_status = nns_proposal_cache_status_usage();

    assert!(nns.contains("proposal"));
    assert!(!nns.contains("\n  proposals"));
    assert!(proposal.contains("list"));
    assert!(proposal.contains("info"));
    assert!(proposal.contains("refresh"));
    assert!(proposal.contains("cache"));
    assert!(proposal.contains("icq nns proposal list"));
    assert!(proposal.contains("icq nns proposal info 132411"));
    assert!(proposal.contains("icq nns proposal refresh"));
    assert!(proposal.contains("icq nns proposal cache status"));
    assert!(proposal_list.contains("icq nns proposal list"));
    assert!(proposal_list.contains("--reward-status settled"));
    assert!(proposal_info.contains("icq nns proposal info 132411"));
    assert!(proposal_refresh.contains("icq nns proposal refresh"));
    assert!(proposal_refresh.contains("--page-size"));
    assert!(proposal_refresh.contains("--max-pages"));
    assert!(proposal_cache.contains("icq nns proposal cache list"));
    assert!(proposal_cache.contains("icq nns proposal cache status"));
    assert!(proposal_cache_list.contains("icq nns proposal cache list"));
    assert!(proposal_cache_status.contains("icq nns proposal cache status"));
    assert!(proposal_list.contains("--limit 50"));
    assert!(proposal_list.contains("--before 132000"));
    assert!(proposal_list.contains("--status open"));
    assert!(proposal_list.contains("--reward-status settled"));
    assert!(proposal_list.contains("--topic governance"));
    assert!(proposal_list.contains("--sort title --asc"));
    assert!(!proposal.contains("icq nns proposal 132411"));
    assert!(proposal_info.contains("--ballots"));
    assert!(proposal_info.contains("--verbose"));
    assert!(proposal_info.contains("--format json"));
}

#[test]
fn nns_proposal_list_rejects_direction_without_local_sort() {
    let err = NnsProposalListOptions::parse_list([OsString::from("--desc")])
        .expect_err("direction without local sort rejected");

    assert!(err.to_string().contains("--desc requires --sort"));
}

#[test]
fn nns_proposal_list_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("proposal"),
        OsString::from("list"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns proposal list"));
}

#[test]
fn nns_proposals_alias_is_rejected() {
    let err = run([
        OsString::from("proposals"),
        OsString::from("--limit"),
        OsString::from("10"),
    ])
    .expect_err("old proposals alias rejected");

    assert!(err.to_string().contains("Usage: icq nns"));
}

#[test]
fn nns_proposal_bare_id_alias_is_rejected() {
    let err = run([OsString::from("proposal"), OsString::from("132411")])
        .expect_err("bare proposal id alias rejected");

    assert!(err.to_string().contains("Usage: icq nns proposal"));
}
