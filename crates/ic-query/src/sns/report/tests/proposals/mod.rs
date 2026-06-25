use super::{fixtures::*, *};
use crate::test_support::temp_dir;
use std::fs;

mod cache_status;
mod cached_core;
mod cached_filters;
mod cached_sorts;
mod live;

fn refresh_fixture_sns_proposals_cache(root: &std::path::Path) -> std::path::PathBuf {
    let request = sns_proposals_refresh_request(root, None);
    let refresh = refresh_sns_proposals_cache_with_source(&request, &FixtureSnsProposalsSource)
        .expect("refresh proposals");
    std::path::PathBuf::from(refresh.cache_path)
}

fn assert_invalid_sns_proposals_cache_status(root: &std::path::Path, expected_error: &str) {
    let status = build_sns_proposals_cache_status_report(&SnsProposalsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.to_path_buf(),
        input: ROOT_A.to_string(),
    })
    .expect("proposal cache status");
    let status_text = sns_proposals_cache_status_report_text(&status);
    let cache = status.cache.as_ref().expect("cache summary");

    assert!(status.found);
    assert_eq!(cache.cache_status, "invalid");
    assert!(
        cache
            .cache_error
            .as_ref()
            .is_some_and(|error| error.contains(expected_error))
    );
    assert!(status_text.contains("cache_status: invalid"));
    assert!(status_text.contains("cache_error:"));

    let list = build_sns_proposals_cache_list_report(&SnsProposalsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.to_path_buf(),
    })
    .expect("proposal cache list");
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].cache_status, "invalid");
    assert!(
        list.caches[0]
            .cache_error
            .as_ref()
            .is_some_and(|error| error.contains(expected_error))
    );
}

struct UnsortedSnsProposalsSource;

impl SnsListSource for UnsortedSnsProposalsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsProposalsSource for UnsortedSnsProposalsSource {
    fn fetch_sns_proposals(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _include_status: &[i32],
        _topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError> {
        Err(SnsHostError::AgentCall {
            method: "fetch_sns_proposals",
            reason: "unexpected bounded live proposal call".to_string(),
        })
    }

    fn fetch_sns_proposal_page(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> Result<MainnetSnsProposalPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 100);
        assert_eq!(before_proposal_id, None);
        Ok(MainnetSnsProposalPage {
            proposals: vec![
                proposal_row_with_fixture(ProposalRowFixture {
                    proposal_id: 10,
                    created_at_secs: 1_700_000_100,
                    decided_at_secs: Some(1_700_001_100),
                    executed_at_secs: Some(1_700_002_300),
                    failed_at_secs: Some(1_700_003_100),
                    decision_state: SNS_PROPOSAL_DECISION_EXECUTED,
                    status: SNS_PROPOSAL_STATUS_EXECUTED_CODE,
                    topic: SnsProposalTopicFilter::Governance,
                    title: "Zulu proposal",
                    action: "motion",
                    action_id: 2,
                    tally: (1_700_005_100, 90, 10, 100),
                    ballot_count: 4,
                    reject_cost_e8s: 100_000_000,
                    reward_event_round: 7,
                    reward_event_end_timestamp_seconds: Some(1_700_004_100),
                    is_eligible_for_rewards: false,
                    proposer_neuron_id: Some("bbbb"),
                }),
                proposal_row_with_fixture(ProposalRowFixture {
                    proposal_id: 20,
                    created_at_secs: 1_700_000_300,
                    decided_at_secs: None,
                    executed_at_secs: None,
                    failed_at_secs: None,
                    decision_state: SNS_PROPOSAL_DECISION_OPEN,
                    status: SNS_PROPOSAL_STATUS_REJECTED_CODE,
                    topic: SnsProposalTopicFilter::TreasuryAssetManagement,
                    title: "Alpha proposal",
                    action: "upgrade-sns-controlled-canister",
                    action_id: 9,
                    tally: (1_700_005_300, 5, 10, 15),
                    ballot_count: 2,
                    reject_cost_e8s: 50_000_000,
                    reward_event_round: 10,
                    reward_event_end_timestamp_seconds: None,
                    is_eligible_for_rewards: true,
                    proposer_neuron_id: None,
                }),
                proposal_row_with_fixture(ProposalRowFixture {
                    proposal_id: 30,
                    created_at_secs: 1_700_000_200,
                    decided_at_secs: Some(1_700_001_300),
                    executed_at_secs: Some(1_700_002_100),
                    failed_at_secs: Some(1_700_003_300),
                    decision_state: SNS_PROPOSAL_DECISION_DECIDED,
                    status: SNS_PROPOSAL_STATUS_ADOPTED_CODE,
                    topic: SnsProposalTopicFilter::Governance,
                    title: "Beta proposal",
                    action: "motion",
                    action_id: 5,
                    tally: (1_700_005_200, 50, 25, 75),
                    ballot_count: 6,
                    reject_cost_e8s: 200_000_000,
                    reward_event_round: 8,
                    reward_event_end_timestamp_seconds: Some(1_700_004_300),
                    is_eligible_for_rewards: true,
                    proposer_neuron_id: Some("aaaa"),
                }),
            ],
            last_cursor: None,
        })
    }
}

fn proposal_ids(report: &SnsProposalsReport) -> Vec<u64> {
    report
        .proposals
        .iter()
        .filter_map(|proposal| proposal.proposal_id)
        .collect()
}

fn assert_cached_proposal_sort(
    sort: SnsProposalsSort,
    direction: SnsProposalSortDirection,
    expected_proposal_ids: &[u64],
) {
    let root = temp_dir(&format!("ic-query-sns-proposals-sort-{}", sort.as_str()));
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = sort;
    request.sort_direction = direction;
    request.limit = 2;

    let report = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh sorted proposals cache");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.sort, sort.as_str());
    assert_eq!(proposal_ids(&report), expected_proposal_ids);

    let _ = fs::remove_dir_all(root);
}

fn assert_cached_status_filter(status: SnsProposalStatusFilter, expected_proposal_ids: &[u64]) {
    let root = temp_dir(&format!(
        "ic-query-sns-proposals-status-{}",
        status.as_str()
    ));
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = status;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh status-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.status_filter, status.as_str());
    assert_eq!(proposal_ids(&first), expected_proposal_ids);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse status-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.status_filter, status.as_str());
    assert_eq!(proposal_ids(&second), expected_proposal_ids);

    let _ = fs::remove_dir_all(root);
}

fn assert_cached_eligibility_filter(
    eligibility: SnsProposalEligibilityFilter,
    expected_proposal_ids: &[u64],
) {
    let root = temp_dir(&format!(
        "ic-query-sns-proposals-eligibility-{}",
        eligibility.as_str()
    ));
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.eligibility = eligibility;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh eligibility-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.eligibility_filter, eligibility.as_str());
    assert_eq!(proposal_ids(&first), expected_proposal_ids);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse eligibility-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.eligibility_filter, eligibility.as_str());
    assert_eq!(proposal_ids(&second), expected_proposal_ids);

    let text = sns_proposals_report_text(&second);
    assert!(text.contains(&format!("eligibility_filter: {}", eligibility.as_str())));

    let _ = fs::remove_dir_all(root);
}

fn assert_cached_proposer_filter(proposer_neuron_id: &str, expected_proposal_ids: &[u64]) {
    let root = temp_dir(&format!(
        "ic-query-sns-proposals-proposer-{proposer_neuron_id}"
    ));
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.proposer_neuron_id = Some(proposer_neuron_id.to_string());
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh proposer-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.proposer_filter.as_deref(), Some(proposer_neuron_id));
    assert_eq!(proposal_ids(&first), expected_proposal_ids);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse proposer-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.proposer_filter.as_deref(), Some(proposer_neuron_id));
    assert_eq!(proposal_ids(&second), expected_proposal_ids);

    let text = sns_proposals_report_text(&second);
    assert!(text.contains(&format!("proposer_filter: {proposer_neuron_id}")));

    let _ = fs::remove_dir_all(root);
}

fn assert_cached_query_filter(query: &str, expected_proposal_ids: &[u64]) {
    let root = temp_dir(&format!("ic-query-sns-proposals-query-{query}"));
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.query = Some(query.to_string());
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh query-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.query_filter.as_deref(), Some(query));
    assert_eq!(proposal_ids(&first), expected_proposal_ids);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse query-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.query_filter.as_deref(), Some(query));
    assert_eq!(proposal_ids(&second), expected_proposal_ids);

    let text = sns_proposals_report_text(&second);
    let json = serde_json::to_value(&second).expect("serialize cached SNS proposals report");
    assert_eq!(json["query_filter"], query);
    assert!(text.contains(&format!("query_filter: {query}")));

    let _ = fs::remove_dir_all(root);
}

fn remove_cached_proposal_status_fields(cache_path: &std::path::Path) {
    remove_cached_proposal_field(cache_path, "status");
}

fn remove_cached_proposal_field(cache_path: &std::path::Path, field: &str) {
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(cache_path).expect("read cache")).expect("parse cache");
    for proposal in cache["proposals"].as_array_mut().expect("cached proposals") {
        proposal
            .as_object_mut()
            .expect("cached proposal object")
            .remove(field);
    }
    fs::write(
        cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize legacy cache"),
    )
    .expect("write legacy cache");
}

fn assert_cached_proposal_status_fields_present(cache_path: &std::path::Path) {
    assert_cached_proposal_field_present(cache_path, "status");
}

fn assert_cached_proposal_field_present(cache_path: &std::path::Path, field: &str) {
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(cache_path).expect("read cache")).expect("parse cache");
    assert!(
        cache["proposals"]
            .as_array()
            .expect("cached proposals")
            .iter()
            .all(|proposal| proposal.get(field).is_some())
    );
}

struct ProposalRowFixture {
    proposal_id: u64,
    created_at_secs: u64,
    decided_at_secs: Option<u64>,
    executed_at_secs: Option<u64>,
    failed_at_secs: Option<u64>,
    decision_state: &'static str,
    status: i32,
    topic: SnsProposalTopicFilter,
    title: &'static str,
    action: &'static str,
    action_id: u64,
    tally: (u64, u64, u64, u64),
    ballot_count: usize,
    reject_cost_e8s: u64,
    reward_event_round: u64,
    reward_event_end_timestamp_seconds: Option<u64>,
    is_eligible_for_rewards: bool,
    proposer_neuron_id: Option<&'static str>,
}

fn proposal_row_with_fixture(fixture: ProposalRowFixture) -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: Some(fixture.proposal_id),
        decision_state: fixture.decision_state.to_string(),
        status: Some(fixture.status),
        topic: Some(fixture.topic.as_str().to_string()),
        title: fixture.title.to_string(),
        action: fixture.action.to_string(),
        action_id: fixture.action_id,
        proposal_creation_timestamp_seconds: fixture.created_at_secs,
        created_at: format_utc_timestamp_secs(fixture.created_at_secs),
        decided_timestamp_seconds: fixture.decided_at_secs,
        decided_at: fixture.decided_at_secs.map(format_utc_timestamp_secs),
        executed_timestamp_seconds: fixture.executed_at_secs,
        executed_at: fixture.executed_at_secs.map(format_utc_timestamp_secs),
        failed_timestamp_seconds: fixture.failed_at_secs,
        failed_at: fixture.failed_at_secs.map(format_utc_timestamp_secs),
        reject_cost_e8s: fixture.reject_cost_e8s,
        ballot_count: fixture.ballot_count,
        reward_event_round: fixture.reward_event_round,
        reward_event_end_timestamp_seconds: fixture.reward_event_end_timestamp_seconds,
        is_eligible_for_rewards: fixture.is_eligible_for_rewards,
        proposer_neuron_id: fixture.proposer_neuron_id.map(ToString::to_string),
        latest_tally: Some(SnsProposalTally {
            timestamp_seconds: fixture.tally.0,
            yes: fixture.tally.1,
            no: fixture.tally.2,
            total: fixture.tally.3,
        }),
        ..fixture_proposal_row()
    }
}
