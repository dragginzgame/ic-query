use super::{fixtures::*, *};
use crate::test_support::temp_dir;
use std::fs;

#[test]
fn sns_proposal_resolves_list_id_and_renders_governance_proposal() {
    let request = proposal_request("1");

    let report = build_sns_proposal_report_with_source(&request, &FixtureSnsProposalSource)
        .expect("sns proposal report");
    let text = sns_proposal_report_text(&report);

    assert_eq!(report.schema_version, SNS_PROPOSAL_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.proposal_id, 42);
    assert!(report.show_ballots);
    assert_eq!(report.proposal.proposal_id, Some(42));
    assert_eq!(report.proposal.action, "motion");
    assert_eq!(report.proposal.decision_state, SNS_PROPOSAL_DECISION_OPEN);
    assert_eq!(report.proposal.ballot_count, 1);
    assert_eq!(report.proposal.ballots[0].vote_text, "yes");
    assert_eq!(report.data_source, "live");
    assert_eq!(report.cache_path, None);
    assert_eq!(report.cache_complete, None);
    assert!(text.contains("proposal_id: 42"));
    assert!(text.contains("data_source: live"));
    assert!(text.contains("action: motion"));
    assert!(text.contains("ballot_count: 1"));
    assert!(text.contains("show_ballots: yes"));
    assert!(text.contains("NEURON_ID   VOTE"));
    assert!(text.contains("00010203"));
    assert!(text.contains("1.00"));
    assert!(text.contains("Fixture proposal"));
    assert!(text.contains("reject_cost: 1.00"));
}

#[test]
fn sns_proposals_resolves_list_id_and_renders_governance_proposals() {
    let request = proposals_request("1");

    let report = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect("sns proposals report");
    let text = sns_proposals_report_text(&report);

    assert_eq!(report.schema_version, SNS_PROPOSALS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.requested_limit, 10);
    assert_eq!(report.before_proposal_id, Some(99));
    assert_eq!(report.status_filter, "open");
    assert_eq!(report.topic_filter, "governance");
    assert_eq!(report.sort, "api");
    assert_eq!(report.sort_direction, "none");
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(42));
    assert_eq!(report.proposals[0].action, "motion");
    assert_eq!(
        report.proposals[0].decision_state,
        SNS_PROPOSAL_DECISION_OPEN
    );
    assert_eq!(report.proposals[0].reject_cost_e8s, 100_000_000);
    assert_eq!(report.proposals[0].created_at, "2026-06-01T00:00:00Z");
    assert_eq!(
        report.proposals[0]
            .latest_tally
            .as_ref()
            .map(|tally| tally.yes),
        Some(10)
    );
    assert_eq!(report.data_source, "live");
    assert_eq!(report.cache_path, None);
    assert_eq!(report.cache_complete, None);
    assert!(text.contains("status_filter: open"));
    assert!(text.contains("data_source: live"));
    assert!(text.contains("topic_filter: governance"));
    assert!(text.contains("sort: api"));
    assert!(text.contains("sort_direction: none"));
    assert!(text.contains("before_proposal_id: 99"));
    assert!(text.contains("proposal_count: 1"));
    assert!(text.contains("ID   ACTION"));
    assert!(text.contains("motion"));
    assert!(text.contains("Fixture proposal"));
}

#[test]
fn sns_proposals_refresh_writes_complete_cache_and_status_reports_it() {
    let root = temp_dir("ic-query-sns-proposals-refresh");
    let request = sns_proposals_refresh_request(&root, None);

    let refresh = refresh_sns_proposals_cache_with_source(&request, &FixtureSnsProposalsSource)
        .expect("refresh proposals");
    let cache_path = std::path::PathBuf::from(&refresh.cache_path);
    let attempt_path = std::path::PathBuf::from(&refresh.refresh_attempt_path);
    let lock_path = std::path::PathBuf::from(&refresh.refresh_lock_path);

    assert!(cache_path.is_file());
    assert!(attempt_path.is_file());
    assert!(!lock_path.exists());
    assert!(refresh.complete);
    assert_eq!(refresh.page_count, 1);
    assert_eq!(refresh.proposal_count, 1);

    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    assert_eq!(cache["schema_version"], 1);
    assert_eq!(cache["domain"], "sns");
    assert_eq!(cache["entity"], ROOT_A);
    assert_eq!(cache["collection"], "proposals");
    assert_eq!(cache["scope"], "full");
    assert_eq!(cache["id"], 1);
    assert_eq!(cache["completeness"]["status"], "api_exhausted");
    assert_eq!(
        cache["proposals"]
            .as_array()
            .expect("cache proposals")
            .len(),
        1
    );
    assert!(cache.get("metadata").is_none());
    assert!(cache.get("data").is_none());

    let list = build_sns_proposals_cache_list_report(&SnsProposalsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("proposal cache list");
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].id, 1);
    assert_eq!(list.caches[0].cache_status, "ok");
    assert_eq!(list.caches[0].cache_error, None);
    assert_eq!(list.caches[0].row_count, 1);

    let status = build_sns_proposals_cache_status_report(&SnsProposalsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
        input: "1".to_string(),
    })
    .expect("proposal cache status");
    let text = sns_proposals_cache_status_report_text(&status);
    assert!(status.found);
    assert_eq!(
        status.cache.as_ref().expect("cache").cache_status.as_str(),
        "ok"
    );
    assert_eq!(
        status
            .cache
            .as_ref()
            .and_then(|cache| cache.latest_attempt.as_ref())
            .map(|attempt| attempt.status.as_str()),
        Some("complete")
    );
    assert!(text.contains("found: yes"));
    assert!(text.contains("row_count: 1"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_missing_cache() {
    let root = temp_dir("ic-query-sns-proposals-cache-missing-status");

    let status = build_sns_proposals_cache_status_report(&SnsProposalsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
        input: ROOT_A.to_string(),
    })
    .expect("proposal cache status");
    let text = sns_proposals_cache_status_report_text(&status);

    assert!(!status.found);
    assert!(status.cache.is_none());
    assert!(status.latest_attempt.is_none());
    assert!(text.contains("found: no"));
    assert!(text.contains("refresh_hint: icq sns proposals refresh"));

    let list = build_sns_proposals_cache_list_report(&SnsProposalsCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("proposal cache list");
    assert_eq!(list.cache_count, 0);
    assert!(list.caches.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_snapshot_identity_mismatch() {
    let root = temp_dir("ic-query-sns-proposals-status-identity-mismatch");
    let cache_path = refresh_fixture_sns_proposals_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["entity"] = serde_json::json!("wrong-root");
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_proposals_cache_status(&root, "identity mismatch");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_unsupported_schema() {
    let root = temp_dir("ic-query-sns-proposals-status-unsupported-schema");
    let cache_path = refresh_fixture_sns_proposals_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["schema_version"] = serde_json::json!(999);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_sns_proposals_cache_status(&root, "unsupported SNS cache schema");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cache_status_reports_malformed_json() {
    let root = temp_dir("ic-query-sns-proposals-status-malformed-json");
    let cache_path = refresh_fixture_sns_proposals_cache(&root);
    fs::write(&cache_path, "{").expect("write malformed cache");

    assert_invalid_sns_proposals_cache_status(&root, "failed to parse SNS cache");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposal_detail_reads_existing_complete_cache_before_live_lookup() {
    let root = temp_dir("ic-query-sns-proposal-detail-cache");
    let refresh_request = sns_proposals_refresh_request(&root, None);
    refresh_sns_proposals_cache_with_source(&refresh_request, &FixtureSnsProposalsSource)
        .expect("refresh proposals cache");
    let mut request = proposal_request("1");
    request.icp_root = Some(root.clone());

    let report = build_sns_proposal_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("cached proposal detail");

    assert_eq!(report.id, 1);
    assert_eq!(report.proposal_id, 42);
    assert_eq!(report.proposal.proposal_id, Some(42));
    assert_eq!(report.proposal.title, "Fixture proposal");
    assert_eq!(report.data_source, "cache");
    assert_eq!(report.cache_complete, Some(true));
    assert!(
        report
            .cache_path
            .as_deref()
            .is_some_and(|path| path.ends_with("/proposals/full.json"))
    );
    let text = sns_proposal_report_text(&report);
    assert!(text.contains("data_source: cache"));
    assert!(text.contains("cache_complete: yes"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_created_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-created");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Created;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "created");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![20, 30]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "created");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![20, 30]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: created"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_created_ascending_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-created-asc");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Created;
    request.sort_direction = SnsProposalSortDirection::Asc;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh ascending sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "created");
    assert_eq!(first.sort_direction, "asc");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse ascending sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "created");
    assert_eq!(second.sort_direction, "asc");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: created"));
    assert!(text.contains("sort_direction: asc"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_decided_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-decided");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Decided;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh decided sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "decided");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse decided sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "decided");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: decided"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_executed_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-executed");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Executed;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh executed sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "executed");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse executed sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "executed");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![10, 30]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: executed"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_failed_orders_before_limit() {
    let root = temp_dir("ic-query-sns-proposals-sort-failed");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Failed;
    request.limit = 2;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh failed sorted proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.sort, "failed");
    assert_eq!(
        first
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse failed sorted proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.sort, "failed");
    assert_eq!(
        second
            .proposals
            .iter()
            .filter_map(|proposal| proposal.proposal_id)
            .collect::<Vec<_>>(),
        vec![30, 10]
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("sort: failed"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_status_decided_filters_complete_snapshot() {
    let root = temp_dir("ic-query-sns-proposals-status-decided");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh decided proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.status_filter, "decided");
    assert_eq!(proposal_ids(&first), vec![30]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse decided proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.status_filter, "decided");
    assert_eq!(proposal_ids(&second), vec![30]);
    assert!(
        second
            .proposals
            .iter()
            .all(|proposal| proposal.decision_state == SNS_PROPOSAL_DECISION_DECIDED)
    );
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("status_filter: decided"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_status_adopted_filters_complete_snapshot() {
    assert_cached_status_filter(SnsProposalStatusFilter::Adopted, &[30]);
}

#[test]
fn sns_proposals_cached_status_rejected_filters_complete_snapshot() {
    assert_cached_status_filter(SnsProposalStatusFilter::Rejected, &[20]);
}

#[test]
fn sns_proposals_status_filter_refreshes_legacy_cache_without_raw_status() {
    let root = temp_dir("ic-query-sns-proposals-status-legacy");
    let refresh = refresh_sns_proposals_cache_with_source(
        &sns_proposals_refresh_request(&root, None),
        &UnsortedSnsProposalsSource,
    )
    .expect("refresh proposals cache");
    let cache_path = std::path::PathBuf::from(refresh.cache_path);
    remove_cached_proposal_status_fields(&cache_path);

    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Adopted;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let report = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("refresh legacy proposals cache before adopted filter");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.status_filter, "adopted");
    assert_eq!(proposal_ids(&report), vec![30]);
    assert_cached_proposal_status_fields_present(&cache_path);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_topic_filters_complete_snapshot() {
    let root = temp_dir("ic-query-sns-proposals-topic-governance");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh topic-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.topic_filter, "governance");
    assert_eq!(proposal_ids(&first), vec![30, 10]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse topic-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.topic_filter, "governance");
    assert_eq!(proposal_ids(&second), vec![30, 10]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_decided_status_combines_with_topic_filter() {
    let root = temp_dir("ic-query-sns-proposals-topic-decided");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let first = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("auto refresh decided topic-filtered proposals cache");

    assert_eq!(first.data_source, "cache");
    assert_eq!(first.status_filter, "decided");
    assert_eq!(first.topic_filter, "governance");
    assert_eq!(proposal_ids(&first), vec![30]);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("reuse decided topic-filtered proposals cache");

    assert_eq!(second.data_source, "cache");
    assert_eq!(second.status_filter, "decided");
    assert_eq!(second.topic_filter, "governance");
    assert_eq!(proposal_ids(&second), vec![30]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_topic_filter_refreshes_legacy_cache_without_topic() {
    let root = temp_dir("ic-query-sns-proposals-topic-legacy");
    let refresh = refresh_sns_proposals_cache_with_source(
        &sns_proposals_refresh_request(&root, None),
        &UnsortedSnsProposalsSource,
    )
    .expect("refresh proposals cache");
    let cache_path = std::path::PathBuf::from(refresh.cache_path);
    remove_cached_proposal_field(&cache_path, "topic");

    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Governance;
    request.before_proposal_id = None;
    request.sort = SnsProposalsSort::Id;
    request.limit = 10;

    let report = build_sns_proposals_report_with_source(&request, &UnsortedSnsProposalsSource)
        .expect("refresh legacy proposals cache before topic filter");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.topic_filter, "governance");
    assert_eq!(proposal_ids(&report), vec![30, 10]);
    assert_cached_proposal_field_present(&cache_path, "topic");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_proposals_cached_sort_title_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Title,
        SnsProposalSortDirection::Asc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_status_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Status,
        SnsProposalSortDirection::Asc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_proposer_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::Proposer,
        SnsProposalSortDirection::Asc,
        &[30, 10],
    );
}

#[test]
fn sns_proposals_cached_sort_total_votes_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::TotalVotes,
        SnsProposalSortDirection::Desc,
        &[10, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_reward_round_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::RewardRound,
        SnsProposalSortDirection::Desc,
        &[20, 30],
    );
}

#[test]
fn sns_proposals_cached_sort_reject_cost_orders_before_limit() {
    assert_cached_proposal_sort(
        SnsProposalsSort::RejectCost,
        SnsProposalSortDirection::Desc,
        &[30, 10],
    );
}

#[test]
fn sns_proposals_status_decided_requires_cache_root() {
    let mut request = proposals_request("1");
    request.status = SnsProposalStatusFilter::Decided;
    request.topic = SnsProposalTopicFilter::Governance;

    let error = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect_err("decided without cache rejected");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedProposalView { .. }
    ));

    request.topic = SnsProposalTopicFilter::Any;
    let error = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect_err("decided without cache rejected");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedProposalView { .. }
    ));
}

#[test]
fn sns_proposals_list_auto_refreshes_missing_cache_and_reuses_it() {
    let root = temp_dir("ic-query-sns-proposals-auto-cache");
    let mut request = proposals_request("1");
    request.icp_root = Some(root.clone());
    request.status = SnsProposalStatusFilter::Any;
    request.topic = SnsProposalTopicFilter::Any;
    request.before_proposal_id = Some(99);
    request.limit = 5;

    let first = build_sns_proposals_report_with_source(&request, &FixtureSnsProposalsSource)
        .expect("auto refresh proposals cache");

    assert_eq!(first.proposal_count, 1);
    assert_eq!(first.proposals[0].proposal_id, Some(42));
    assert_eq!(first.data_source, "cache");
    assert_eq!(first.cache_complete, Some(true));
    assert!(
        first
            .cache_path
            .as_deref()
            .is_some_and(|path| path.ends_with("/proposals/full.json"))
    );

    let status = build_sns_proposals_cache_status_report(&SnsProposalsCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
        input: "1".to_string(),
    })
    .expect("proposal cache status");
    assert!(status.found);

    let second = build_sns_proposals_report_with_source(&request, &NoLiveSnsProposalsSource)
        .expect("cached proposals report");

    assert_eq!(second.proposal_count, 1);
    assert_eq!(second.proposals[0].proposal_id, Some(42));
    assert_eq!(second.source_endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
    assert_eq!(second.data_source, "cache");
    assert_eq!(second.cache_complete, Some(true));
    assert_eq!(second.cache_path, first.cache_path);
    let text = sns_proposals_report_text(&second);
    assert!(text.contains("data_source: cache"));
    assert!(text.contains("cache_complete: yes"));

    let _ = fs::remove_dir_all(root);
}

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
                    tally: (90, 10, 100),
                    ballot_count: 4,
                    reject_cost_e8s: 100_000_000,
                    reward_event_round: 7,
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
                    tally: (5, 10, 15),
                    ballot_count: 2,
                    reject_cost_e8s: 50_000_000,
                    reward_event_round: 10,
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
                    tally: (50, 25, 75),
                    ballot_count: 6,
                    reject_cost_e8s: 200_000_000,
                    reward_event_round: 8,
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
    tally: (u64, u64, u64),
    ballot_count: usize,
    reject_cost_e8s: u64,
    reward_event_round: u64,
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
        proposer_neuron_id: fixture.proposer_neuron_id.map(ToString::to_string),
        latest_tally: Some(SnsProposalTally {
            timestamp_seconds: fixture.created_at_secs,
            yes: fixture.tally.0,
            no: fixture.tally.1,
            total: fixture.tally.2,
        }),
        ..fixture_proposal_row()
    }
}
