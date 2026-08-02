use super::*;
use crate::cli::clap::render_help;
use clap::{ArgMatches, Command as ClapCommand};

fn parse_proposal_matches(
    command: ClapCommand,
    args: &[&str],
) -> Result<ArgMatches, NnsCommandError> {
    parse_nns_matches(command, args.iter().copied().map(std::ffi::OsString::from))
}

fn parse_list_options(args: &[&str]) -> Result<NnsProposalListOptions, NnsCommandError> {
    let matches = parse_proposal_matches(nns_proposal_list_command(), args)?;
    NnsProposalListOptions::from_matches(&matches, MAINNET_NETWORK)
}

#[test]
fn nns_proposal_list_parses_defaults_and_json_format() {
    let defaults = parse_list_options(&[]).expect("parse defaults");

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
    assert_eq!(defaults.proposer_neuron_id, None);
    assert_eq!(defaults.query, None);
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

    let options = parse_list_options(&[
        "--json",
        "--source-endpoint",
        "https://icp-api.io",
        "--limit",
        "50",
        "--before",
        "132000",
        "--status",
        NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
        "--reward-status",
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL,
        "--topic",
        NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
        "--proposer",
        "123456789",
        "--query",
        "subnet",
        "--sort",
        NNS_PROPOSAL_SORT_TITLE_LABEL,
        "--asc",
        "--verbose",
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
    assert_eq!(options.proposer_neuron_id, Some(123_456_789));
    assert_eq!(options.query.as_deref(), Some("subnet"));
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

    let grouped_options = parse_list_options(&[
        "--limit",
        "10",
        "--reward-status",
        NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL,
    ])
    .expect("parse nns proposal list");

    assert_eq!(grouped_options.limit, 10);
    assert_eq!(
        grouped_options.reward_status,
        NnsProposalRewardStatusFilter::Settled
    );
}

#[test]
fn nns_proposal_list_parses_extended_local_sort_values() {
    let reward_status_sort = parse_list_options(&["--sort", NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL])
        .expect("parse reward-status sort");

    assert_eq!(reward_status_sort.sort, NnsProposalListSort::RewardStatus);
    assert_eq!(
        reward_status_sort.sort_direction,
        NnsProposalSortDirection::Asc
    );
    assert_eq!(
        reward_status_sort.sort.as_str(),
        NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL
    );

    let deadline_sort = parse_list_options(&["--sort", NNS_PROPOSAL_SORT_DEADLINE_LABEL])
        .expect("parse deadline sort");

    assert_eq!(deadline_sort.sort, NnsProposalListSort::Deadline);
    assert_eq!(deadline_sort.sort_direction, NnsProposalSortDirection::Desc);

    let tally_time_sort = parse_list_options(&["--sort", NNS_PROPOSAL_SORT_TALLY_TIME_LABEL])
        .expect("parse tally-time sort");

    assert_eq!(tally_time_sort.sort, NnsProposalListSort::TallyTime);
    assert_eq!(
        tally_time_sort.sort_direction,
        NnsProposalSortDirection::Desc
    );
    assert_eq!(
        tally_time_sort.sort.as_str(),
        NNS_PROPOSAL_SORT_TALLY_TIME_LABEL
    );

    let voting_power_sort = parse_list_options(&["--sort", NNS_PROPOSAL_SORT_VOTING_POWER_LABEL])
        .expect("parse voting-power sort");

    assert_eq!(voting_power_sort.sort, NnsProposalListSort::VotingPower);
    assert_eq!(
        voting_power_sort.sort_direction,
        NnsProposalSortDirection::Desc
    );
}

#[test]
fn nns_proposal_parses_id_and_json_format() {
    let options = parse_test_options(
        nns_proposal_info_command(),
        &[
            "132411",
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--ballots",
            "--verbose",
        ],
        NnsProposalOptions::from_matches,
    )
    .expect("parse nns proposal info");

    assert_eq!(options.network, MAINNET_NETWORK);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.proposal_id, 132_411);
    assert!(options.show_ballots);
    assert!(options.verbose);

    let grouped_options = parse_test_options(
        nns_proposal_info_command(),
        &["132411", "--ballots", "--verbose"],
        NnsProposalOptions::from_matches,
    )
    .expect("parse nns proposal info");

    assert_eq!(grouped_options.proposal_id, 132_411);
    assert!(grouped_options.show_ballots);
    assert!(grouped_options.verbose);
}

#[test]
fn nns_proposal_refresh_parses_cache_options() {
    let defaults = parse_test_options(
        nns_proposal_refresh_command(),
        &[],
        NnsProposalRefreshOptions::from_matches,
    )
    .expect("parse refresh defaults");

    assert_eq!(defaults.network, MAINNET_NETWORK);
    assert_eq!(defaults.format, OutputFormat::Text);
    assert_eq!(
        defaults.source_endpoint,
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT
    );
    assert_eq!(defaults.page_size, 100);
    assert_eq!(defaults.max_pages, None);

    let options = parse_test_options(
        nns_proposal_refresh_command(),
        &[
            "--json",
            "--source-endpoint",
            "https://icp-api.io",
            "--page-size",
            "25",
            "--max-pages",
            "2",
        ],
        NnsProposalRefreshOptions::from_matches,
    )
    .expect("parse refresh options");

    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, "https://icp-api.io");
    assert_eq!(options.page_size, 25);
    assert_eq!(options.max_pages, Some(2));
}

#[test]
fn nns_proposal_cache_options_parse_json_format() {
    let list = parse_test_options(
        nns_proposal_cache_list_command(),
        &["--json"],
        NnsProposalCacheOptions::from_matches,
    )
    .expect("parse cache list");
    let status = parse_test_options(
        nns_proposal_cache_status_command(),
        &["--json"],
        NnsProposalCacheOptions::from_matches,
    )
    .expect("parse cache status");

    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.format, OutputFormat::Json);
    assert_eq!(status.network, MAINNET_NETWORK);
    assert_eq!(status.format, OutputFormat::Json);
}

#[test]
fn nns_proposal_help_is_advertised_under_nns() {
    let nns = usage();
    let proposal = render_help(nns_proposal_command());
    let proposal_list = render_help(nns_proposal_list_command());
    let proposal_info = render_help(nns_proposal_info_command());
    let proposal_refresh = render_help(nns_proposal_refresh_command());
    let proposal_cache = render_help(nns_proposal_cache_command());
    let proposal_cache_list = render_help(nns_proposal_cache_list_command());
    let proposal_cache_status = render_help(nns_proposal_cache_status_command());

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
    assert!(proposal_list.contains("Collection mode: Cache-preferred read"));
    assert!(proposal_list.contains("--reward-status settled"));
    assert!(proposal_info.contains("icq nns proposal info 132411"));
    assert!(proposal_refresh.contains("icq nns proposal refresh"));
    assert!(proposal_refresh.contains("Collection mode: Forced live refresh"));
    assert!(proposal_refresh.contains("--page-size"));
    assert!(proposal_refresh.contains("--max-pages"));
    assert!(proposal_cache.contains("icq nns proposal cache list"));
    assert!(proposal_cache.contains("icq nns proposal cache status"));
    assert!(proposal_cache_list.contains("icq nns proposal cache list"));
    assert!(proposal_cache_status.contains("icq nns proposal cache status"));
    assert!(proposal_cache_list.contains("Collection mode: Local cache inspection"));
    assert!(proposal_cache_status.contains("Collection mode: Local cache inspection"));
    assert!(!proposal_cache_list.contains("--source-endpoint"));
    assert!(!proposal_cache_status.contains("--source-endpoint"));
    assert!(proposal_list.contains("--limit 50"));
    assert!(proposal_list.contains("--before 132000"));
    assert!(proposal_list.contains("--status open"));
    assert!(proposal_list.contains("--reward-status settled"));
    assert!(proposal_list.contains("--topic governance"));
    assert!(proposal_list.contains("--sort reward-status"));
    assert!(proposal_list.contains("--sort deadline"));
    assert!(proposal_list.contains("--sort voting-power"));
    assert!(proposal_list.contains("--sort title --asc"));
    assert!(!proposal.contains("icq nns proposal 132411"));
    assert!(proposal_info.contains("--ballots"));
    assert!(proposal_info.contains("--verbose"));
    assert!(proposal_info.contains("--json"));
}

#[test]
fn nns_proposal_list_rejects_direction_without_local_sort() {
    let err = parse_list_options(&["--desc"]).expect_err("direction without local sort rejected");

    assert!(err.to_string().contains("--desc requires --sort"));
}
