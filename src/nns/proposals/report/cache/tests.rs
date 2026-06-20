use super::{
    model::{
        NnsProposalCacheListRequest, NnsProposalCacheStatusRequest, NnsProposalRefreshRequest,
    },
    refresh::refresh_nns_proposal_cache_with_source,
    reports::{
        build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
        build_nns_proposal_list_report_from_cache, build_nns_proposal_report_from_cache,
    },
};
use crate::{
    ic_registry::{DEFAULT_MAINNET_ENDPOINT, MAINNET_GOVERNANCE_CANISTER_ID},
    nns::proposals::report::{
        NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION, NNS_PROPOSAL_REPORT_SCHEMA_VERSION,
        NnsProposalHostError, NnsProposalListRequest, NnsProposalRequest,
        cache::paths::nns_proposal_cache_paths,
        model::{
            NNS_PROPOSAL_SORT_ASC_LABEL, NNS_PROPOSAL_SORT_TITLE_LABEL,
            NNS_PROPOSAL_STATUS_EXECUTED_LABEL, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
            NnsProposalListSort, NnsProposalRewardStatusFilter, NnsProposalSortDirection,
            NnsProposalStatusFilter, NnsProposalTopicFilter,
        },
        source::{NnsProposalFetchRequest, NnsProposalSource},
        text::{
            nns_proposal_cache_status_report_text, nns_proposal_list_report_text,
            nns_proposal_refresh_report_text, nns_proposal_report_text,
        },
        wire::{
            NnsGovernanceBallot, NnsNeuronId, NnsProposal, NnsProposalAction, NnsProposalId,
            NnsProposalInfo, NnsProposalTallyWire,
        },
    },
    subnet_catalog::MAINNET_NETWORK,
    test_support::temp_dir,
};
use candid::Reserved;
use std::fs;

struct FixtureSource;

impl NnsProposalSource for FixtureSource {
    fn fetch_proposals(
        &self,
        _request: &NnsProposalFetchRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        include_reward_status: &[i32],
    ) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
        assert_eq!(limit, 2);
        assert!(include_status.is_empty());
        assert!(include_reward_status.is_empty());
        Ok(match before_proposal_id {
            None => vec![proposal_info(3), proposal_info(2)],
            Some(2) => vec![proposal_info(1)],
            other => panic!("unexpected before proposal id: {other:?}"),
        })
    }

    fn fetch_proposal(
        &self,
        _request: &NnsProposalFetchRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalInfo, NnsProposalHostError> {
        Ok(proposal_info(proposal_id))
    }
}

#[test]
fn nns_proposal_refresh_writes_complete_cache_and_status_reports() {
    let root = temp_dir("ic-query-nns-proposal-cache");
    let request = NnsProposalRefreshRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        icp_root: root.clone(),
        page_size: 2,
        max_pages: None,
    };

    let report =
        refresh_nns_proposal_cache_with_source(&request, &FixtureSource).expect("refresh cache");
    let refresh_text = nns_proposal_refresh_report_text(&report);

    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(
        report.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(report.proposal_count, 3);
    assert_eq!(report.page_count, 2);
    assert!(report.complete);
    assert!(report.wrote_cache);
    assert!(!report.replaced_existing_cache);
    assert!(refresh_text.contains("proposal_count: 3"));
    let cache_path = nns_proposal_cache_paths(&root, MAINNET_NETWORK).snapshot_path;
    assert!(cache_path.is_file());
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(cache_path).expect("read cache")).expect("parse cache");
    assert_eq!(cache["domain"], "nns");
    assert_eq!(cache["entity"], "governance");
    assert_eq!(cache["collection"], "proposals");
    assert_eq!(cache["scope"], "full");

    let list = build_nns_proposal_cache_list_report(&NnsProposalCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("cache list");

    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].cache_status, "ok");
    assert_eq!(list.caches[0].cache_error, None);
    assert_eq!(list.caches[0].row_count, 3);
    assert_eq!(list.caches[0].page_count, 2);

    let status = build_nns_proposal_cache_status_report(&NnsProposalCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root,
    })
    .expect("cache status");
    let status_text = nns_proposal_cache_status_report_text(&status);

    assert!(status.found);
    assert_eq!(
        status.cache.as_ref().expect("cache").cache_status.as_str(),
        "ok"
    );
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .expect("latest attempt")
            .status,
        "complete"
    );
    assert!(status_text.contains("latest_attempt:"));
    assert!(status_text.contains("status: complete"));
}

#[test]
fn nns_proposal_cache_status_reports_missing_cache() {
    let root = temp_dir("ic-query-nns-proposal-status-missing");

    let status = build_nns_proposal_cache_status_report(&NnsProposalCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("cache status");
    let text = nns_proposal_cache_status_report_text(&status);

    assert!(!status.found);
    assert!(status.cache.is_none());
    assert!(status.latest_attempt.is_none());
    assert!(text.contains("found: no"));
    assert!(text.contains("refresh_hint: icq nns proposal refresh"));

    let list = build_nns_proposal_cache_list_report(&NnsProposalCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("cache list");
    assert_eq!(list.cache_count, 0);
    assert!(list.caches.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn nns_proposal_list_reads_existing_complete_cache_before_live_lookup() {
    let root = temp_dir("ic-query-nns-proposal-list-cache");
    refresh_nns_proposal_cache_with_source(
        &NnsProposalRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            icp_root: root.clone(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
    )
    .expect("refresh cache");

    let request = NnsProposalListRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_100_000,
        limit: 1,
        before_proposal_id: Some(3),
        status: NnsProposalStatusFilter::Executed,
        reward_status: NnsProposalRewardStatusFilter::Settled,
        topic: NnsProposalTopicFilter::Governance,
        proposer_neuron_id: Some(99),
        sort: NnsProposalListSort::Title,
        sort_direction: NnsProposalSortDirection::Asc,
        verbose: false,
    };
    let report = build_nns_proposal_list_report_from_cache(&request, &root)
        .expect("cache lookup")
        .expect("cached list report");
    let text = nns_proposal_list_report_text(&report);

    assert_eq!(
        report.schema_version,
        NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report.data_source, "cache");
    assert!(report.cache_complete.expect("cache completeness"));
    assert_eq!(report.status_filter, NNS_PROPOSAL_STATUS_EXECUTED_LABEL);
    assert_eq!(report.topic_filter, NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL);
    assert_eq!(report.proposer_filter, Some(99));
    assert_eq!(report.sort, NNS_PROPOSAL_SORT_TITLE_LABEL);
    assert_eq!(report.sort_direction, NNS_PROPOSAL_SORT_ASC_LABEL);
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, Some(1));
    assert!(text.contains("data_source: cache"));
    assert!(text.contains("cache_complete: yes"));
}

#[test]
fn nns_proposal_list_cache_lookup_returns_none_when_cache_is_missing() {
    let root = temp_dir("ic-query-nns-proposal-list-cache-missing");
    let report = build_nns_proposal_list_report_from_cache(
        &NnsProposalListRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_100_000,
            limit: 25,
            before_proposal_id: None,
            status: NnsProposalStatusFilter::Any,
            reward_status: NnsProposalRewardStatusFilter::Any,
            topic: NnsProposalTopicFilter::Any,
            proposer_neuron_id: None,
            sort: NnsProposalListSort::Api,
            sort_direction: NnsProposalSortDirection::Desc,
            verbose: false,
        },
        &root,
    )
    .expect("cache lookup");

    assert!(report.is_none());
}

#[test]
fn nns_proposal_cache_status_reports_snapshot_identity_mismatch() {
    let root = temp_dir("ic-query-nns-proposal-identity-mismatch");
    let cache_path = refresh_fixture_nns_proposal_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["entity"] = serde_json::json!("wrong-governance");
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_nns_proposal_cache_status(&root, "identity mismatch");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn nns_proposal_cache_status_reports_unsupported_schema() {
    let root = temp_dir("ic-query-nns-proposal-unsupported-schema");
    let cache_path = refresh_fixture_nns_proposal_cache(&root);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["schema_version"] = serde_json::json!(999);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_nns_proposal_cache_status(&root, "unsupported NNS proposal cache schema");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn nns_proposal_cache_status_reports_malformed_json() {
    let root = temp_dir("ic-query-nns-proposal-malformed-json");
    let cache_path = refresh_fixture_nns_proposal_cache(&root);
    fs::write(&cache_path, "{").expect("write malformed cache");

    assert_invalid_nns_proposal_cache_status(&root, "failed to parse NNS proposal cache");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn nns_proposal_detail_reads_existing_complete_cache_before_live_lookup() {
    let root = temp_dir("ic-query-nns-proposal-detail-cache");
    refresh_nns_proposal_cache_with_source(
        &NnsProposalRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            icp_root: root.clone(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
    )
    .expect("refresh cache");

    let request = NnsProposalRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_100_000,
        proposal_id: 2,
        show_ballots: true,
        verbose: false,
    };
    let report = build_nns_proposal_report_from_cache(&request, &root)
        .expect("cache lookup")
        .expect("cached proposal report");
    let text = nns_proposal_report_text(&report);

    assert_eq!(report.schema_version, NNS_PROPOSAL_REPORT_SCHEMA_VERSION);
    assert_eq!(report.proposal_id, 2);
    assert_eq!(report.proposal.title.as_deref(), Some("Proposal 2"));
    assert_eq!(report.data_source, "cache");
    assert!(report.cache_complete.expect("cache completeness"));
    assert!(
        report
            .cache_path
            .as_deref()
            .expect("cache path")
            .ends_with(".icq/nns/ic/governance/proposals/full.json")
    );
    assert!(text.contains("data_source: cache"));
    assert!(text.contains("cache_complete: yes"));
    assert!(text.contains("cache_path: "));
}

#[test]
fn nns_proposal_detail_cache_lookup_returns_none_for_missing_cached_proposal() {
    let root = temp_dir("ic-query-nns-proposal-detail-cache-missing");
    refresh_nns_proposal_cache_with_source(
        &NnsProposalRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            icp_root: root.clone(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
    )
    .expect("refresh cache");

    let report = build_nns_proposal_report_from_cache(
        &NnsProposalRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_100_000,
            proposal_id: 42,
            show_ballots: false,
            verbose: false,
        },
        &root,
    )
    .expect("cache lookup");

    assert!(report.is_none());
}

fn refresh_fixture_nns_proposal_cache(root: &std::path::Path) -> std::path::PathBuf {
    refresh_nns_proposal_cache_with_source(
        &NnsProposalRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            icp_root: root.to_path_buf(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
    )
    .expect("refresh cache");
    nns_proposal_cache_paths(root, MAINNET_NETWORK).snapshot_path
}

fn assert_invalid_nns_proposal_cache_status(root: &std::path::Path, expected_error: &str) {
    let status = build_nns_proposal_cache_status_report(&NnsProposalCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.to_path_buf(),
    })
    .expect("cache status");
    let status_text = nns_proposal_cache_status_report_text(&status);
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

    let list = build_nns_proposal_cache_list_report(&NnsProposalCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.to_path_buf(),
    })
    .expect("cache list");
    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].cache_status, "invalid");
    assert!(
        list.caches[0]
            .cache_error
            .as_ref()
            .is_some_and(|error| error.contains(expected_error))
    );
}

fn proposal_info(proposal_id: u64) -> NnsProposalInfo {
    NnsProposalInfo {
        id: Some(NnsProposalId { id: proposal_id }),
        status: 4,
        topic: 4,
        ballots: vec![(
            proposal_id,
            NnsGovernanceBallot {
                vote: 1,
                voting_power: 100,
            },
        )],
        proposal_timestamp_seconds: 1_700_000_000 + proposal_id,
        reward_event_round: 7,
        deadline_timestamp_seconds: Some(1_700_010_000),
        failed_timestamp_seconds: 0,
        reject_cost_e8s: 100_000_000,
        latest_tally: Some(NnsProposalTallyWire {
            no: 1,
            yes: 2,
            total: 3,
            timestamp_seconds: 1_700_000_100,
        }),
        reward_status: 3,
        decided_timestamp_seconds: 1_700_000_200,
        proposal: Some(NnsProposal {
            url: format!("https://dashboard.internetcomputer.org/proposal/{proposal_id}"),
            title: Some(format!("Proposal {proposal_id}")),
            action: Some(NnsProposalAction::Motion(Reserved)),
            summary: "Proposal summary".to_string(),
        }),
        proposer: Some(NnsNeuronId { id: 99 }),
        executed_timestamp_seconds: 1_700_000_300,
        total_potential_voting_power: Some(100),
    }
}
