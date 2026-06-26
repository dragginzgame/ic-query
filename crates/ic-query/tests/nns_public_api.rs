use ic_query::nns::proposals::{
    NnsProposalBallotRow, NnsProposalListReport, NnsProposalListRequest, NnsProposalListSort,
    NnsProposalReport, NnsProposalRequest, NnsProposalRewardStatusFilter, NnsProposalRow,
    NnsProposalSortDirection, NnsProposalStatusFilter, NnsProposalTally, NnsProposalTopicFilter,
    nns_proposal_list_report_text, nns_proposal_report_text,
};
use ic_query::nns::registry::{
    NnsRegistryVersionReport, NnsRegistryVersionRequest, nns_registry_version_report_text,
};

#[test]
fn public_nns_registry_api_is_constructible_and_renderable() {
    let request = NnsRegistryVersionRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
    };

    assert_eq!(request.network, "ic");

    let report = NnsRegistryVersionReport {
        schema_version: 1,
        network: request.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
    };

    let text = nns_registry_version_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("registry_version: 42"));
}

#[test]
fn public_nns_proposal_api_is_constructible_and_renderable() {
    let request = NnsProposalListRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        limit: 25,
        before_proposal_id: Some(132_500),
        status: NnsProposalStatusFilter::Executed,
        reward_status: NnsProposalRewardStatusFilter::Settled,
        topic: NnsProposalTopicFilter::Governance,
        proposer_neuron_id: Some(12_345),
        query: Some("subnet".to_string()),
        sort: NnsProposalListSort::TallyTime,
        sort_direction: NnsProposalSortDirection::Desc,
        verbose: true,
    };

    assert_eq!(request.status.as_str(), "executed");
    assert_eq!(request.reward_status.as_str(), "settled");
    assert_eq!(request.topic.as_str(), "governance");
    assert_eq!(request.sort.direction_label(request.sort_direction), "desc");

    let proposal = sample_nns_proposal_row();
    let list_report = NnsProposalListReport {
        schema_version: 3,
        network: request.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_source: "cache".to_string(),
        cache_path: Some(".icq/nns/ic/governance/proposals/full.json".to_string()),
        cache_complete: Some(true),
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status_filter: request.status.as_str().to_string(),
        reward_status_filter: request.reward_status.as_str().to_string(),
        topic_filter: request.topic.as_str().to_string(),
        proposer_filter: request.proposer_neuron_id,
        query_filter: request.query,
        sort: request.sort.as_str().to_string(),
        sort_direction: request
            .sort
            .direction_label(request.sort_direction)
            .to_string(),
        result_scope: "complete-cache".to_string(),
        verbose: request.verbose,
        proposal_count: 1,
        proposals: vec![proposal.clone()],
    };

    let list_text = nns_proposal_list_report_text(&list_report);

    assert!(list_text.contains("proposal_count: 1"));
    assert!(list_text.contains("topic_filter: governance"));
    assert!(list_text.contains("proposal_details:"));
    assert!(list_text.contains("title: Upgrade subnet"));

    let detail_request = NnsProposalRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        proposal_id: 132_411,
        show_ballots: true,
        verbose: false,
    };
    let detail_report = NnsProposalReport {
        schema_version: 1,
        network: detail_request.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: detail_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_source: "live".to_string(),
        cache_path: None,
        cache_complete: None,
        proposal_id: detail_request.proposal_id,
        show_ballots: detail_request.show_ballots,
        verbose: detail_request.verbose,
        proposal,
    };

    let detail_text = nns_proposal_report_text(&detail_report);

    assert!(detail_text.contains("proposal_id: 132411"));
    assert!(detail_text.contains("show_ballots: yes"));
    assert!(detail_text.contains("reject_cost: 1.00"));
    assert!(detail_text.contains("ballots:"));
    assert!(detail_text.contains("yes"));
}

fn sample_nns_proposal_row() -> NnsProposalRow {
    NnsProposalRow {
        proposal_id: Some(132_411),
        proposer_neuron_id: Some(12_345),
        topic: 4,
        topic_text: "governance".to_string(),
        status: 4,
        status_text: "executed".to_string(),
        reward_status: 3,
        reward_status_text: "settled".to_string(),
        title: Some("Upgrade subnet".to_string()),
        summary: "Upgrade subnet replica version.".to_string(),
        url: "https://dashboard.internetcomputer.org/proposal/132411".to_string(),
        action_text: Some("execute-nns-function".to_string()),
        reject_cost_e8s: 100_000_000,
        proposal_timestamp_seconds: 1_700_000_000,
        proposed_at: "2023-11-14T22:13:20Z".to_string(),
        deadline_timestamp_seconds: Some(1_700_086_400),
        deadline_at: Some("2023-11-15T22:13:20Z".to_string()),
        decided_timestamp_seconds: 1_700_010_000,
        decided_at: Some("2023-11-15T01:00:00Z".to_string()),
        executed_timestamp_seconds: 1_700_020_000,
        executed_at: Some("2023-11-15T03:46:40Z".to_string()),
        failed_timestamp_seconds: 0,
        failed_at: None,
        reward_event_round: 42,
        total_potential_voting_power: Some(1_000_000_000),
        latest_tally: Some(NnsProposalTally {
            timestamp_seconds: 1_700_010_000,
            yes: 900_000_000,
            no: 100_000_000,
            total: 1_000_000_000,
        }),
        ballot_count: 1,
        ballots: vec![NnsProposalBallotRow {
            neuron_id: 12_345,
            vote: 1,
            vote_text: "yes".to_string(),
            voting_power: 100_000_000,
        }],
    }
}
