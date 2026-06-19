use super::*;

#[test]
fn nns_proposals_parses_defaults_and_json_format() {
    let defaults = NnsProposalsOptions::parse([]).expect("parse defaults");

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
    assert_eq!(defaults.sort, NnsProposalsSort::Api);
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

    let options = NnsProposalsOptions::parse([
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
    .expect("parse nns proposals");

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
    assert_eq!(options.sort, NnsProposalsSort::Title);
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
}

#[test]
fn nns_proposal_parses_id_and_json_format() {
    let options = NnsProposalOptions::parse([
        OsString::from("132411"),
        OsString::from("--format"),
        OsString::from("json"),
        OsString::from("--source-endpoint"),
        OsString::from("https://icp-api.io"),
        OsString::from("--ballots"),
        OsString::from("--verbose"),
    ])
    .expect("parse nns proposal");

    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.proposal_id, 132_411);
    assert!(options.show_ballots);
    assert!(options.verbose);
}

#[test]
fn nns_proposal_help_is_advertised_under_nns() {
    let nns = usage();
    let proposals = nns_proposals_usage();
    let proposal = nns_proposal_usage();

    assert!(nns.contains("proposal"));
    assert!(nns.contains("proposals"));
    assert!(proposals.contains("icq nns proposals"));
    assert!(proposals.contains("--limit 50"));
    assert!(proposals.contains("--before 132000"));
    assert!(proposals.contains("--status open"));
    assert!(proposals.contains("--reward-status settled"));
    assert!(proposals.contains("--topic governance"));
    assert!(proposals.contains("--sort title --asc"));
    assert!(proposal.contains("icq nns proposal 132411"));
    assert!(proposal.contains("--ballots"));
    assert!(proposal.contains("--verbose"));
    assert!(proposal.contains("--format json"));
}

#[test]
fn nns_proposals_rejects_direction_without_local_sort() {
    let err = NnsProposalsOptions::parse([OsString::from("--desc")])
        .expect_err("direction without local sort rejected");

    assert!(err.to_string().contains("--desc requires --sort"));
}

#[test]
fn nns_proposals_local_is_rejected_with_pinned_message() {
    let err = run([
        OsString::from("proposals"),
        OsString::from("--__icq-network"),
        OsString::from("local"),
    ])
    .expect_err("local rejected");

    let message = err.to_string();
    assert!(message.contains("supports only the mainnet `ic` network"));
    assert!(message.contains("icq --network ic nns proposals"));
}
