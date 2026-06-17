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
    assert_eq!(report.proposal.decision_state, "open");
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
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(42));
    assert_eq!(report.proposals[0].action, "motion");
    assert_eq!(report.proposals[0].decision_state, "open");
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
