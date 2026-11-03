use super::refresh::refresh_nns_proposal_cache_with_source_and_progress;
use super::reports::build_nns_proposal_activity_report_from_cache;
use super::reports::build_nns_proposal_cache_list_report;
use super::reports::build_nns_proposal_cache_status_report;
use super::reports::build_nns_proposal_list_report_from_cache;
use super::reports::build_nns_proposal_list_report_with_cache_and_source;
use super::reports::build_nns_proposal_report_from_cache;
use super::reports::build_nns_proposal_report_with_cache_and_source;

use crate::HostCacheError;
use crate::cache::CacheRefreshAttemptStatus;
use crate::cache::CacheValidationStatus;
use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use crate::nns::MAINNET_GOVERNANCE_CANISTER_ID;
use crate::nns::NnsGovernanceCacheRequest;
use crate::nns::NnsGovernanceRefreshRequest;
use crate::nns::governance::NnsGovernanceNeuronId;
use crate::nns::governance::NnsGovernanceProposalId;
use crate::nns::governance::NnsGovernanceRequest;
use crate::nns::governance::NnsGovernanceSourceData;
use crate::nns::governance::NnsGovernanceSourceProvenance;
use crate::nns::governance::NnsGovernanceSourceSelection;
use crate::nns::proposals::report::NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION;
use crate::nns::proposals::report::NNS_PROPOSAL_REPORT_SCHEMA_VERSION;
use crate::nns::proposals::report::NnsProposalActivityError;
use crate::nns::proposals::report::NnsProposalActivityRequest;
use crate::nns::proposals::report::NnsProposalError;
use crate::nns::proposals::report::NnsProposalHostError;
use crate::nns::proposals::report::NnsProposalListRequest;
use crate::nns::proposals::report::NnsProposalRequest;
use crate::nns::proposals::report::cache::paths::nns_proposal_cache_paths;
use crate::nns::proposals::report::model::NnsProposalListSort;
use crate::nns::proposals::report::model::NnsProposalRewardStatusFilter;
use crate::nns::proposals::report::model::NnsProposalRow;
use crate::nns::proposals::report::model::NnsProposalStatus;
use crate::nns::proposals::report::model::NnsProposalStatusFilter;
use crate::nns::proposals::report::model::NnsProposalTopic;
use crate::nns::proposals::report::model::NnsProposalTopicFilter;
use crate::nns::proposals::report::source::NnsProposalSource;
use crate::nns::proposals::report::source::NnsProposalSourceFuture;
use crate::nns::proposals::report::source::nns_proposal_row_from_info;
use crate::nns::proposals::report::wire::NnsGovernanceBallot;
use crate::nns::proposals::report::wire::NnsProposal;
use crate::nns::proposals::report::wire::NnsProposalAction;
use crate::nns::proposals::report::wire::NnsProposalInfo;
use crate::nns::proposals::report::wire::NnsProposalTallyWire;
use crate::report::SortDirection;
use crate::subnet_catalog::MAINNET_NETWORK;
use crate::test_support::temp_dir;
use ic_platform::candid::contract::Reserved;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

struct FixtureSource;

#[tokio::test(flavor = "current_thread")]
async fn cache_aware_proposal_builders_preserve_fallback_and_strict_cache_errors() {
    let root = temp_dir("ic-query-nns-proposal-cache-source");
    let governance = proposal_governance_request(1_700_000_001);
    let list = NnsProposalListRequest::new(governance.clone(), 2);
    let info = NnsProposalRequest::new(governance.clone(), 2);
    assert_eq!(
        build_nns_proposal_list_report_with_cache_and_source(&list, &root, &FixtureSource)
            .await
            .unwrap()
            .provenance
            .data_source()
            .as_str(),
        "live"
    );
    assert_eq!(
        build_nns_proposal_report_with_cache_and_source(&info, &root, &FixtureSource)
            .await
            .unwrap()
            .provenance
            .data_source()
            .as_str(),
        "live"
    );
    let path = refresh_fixture_nns_proposal_cache(&root).await;
    assert_eq!(
        build_nns_proposal_list_report_with_cache_and_source(&list, &root, &FixtureSource)
            .await
            .unwrap()
            .provenance
            .data_source()
            .as_str(),
        "cache"
    );
    assert_eq!(
        build_nns_proposal_report_with_cache_and_source(&info, &root, &FixtureSource)
            .await
            .unwrap()
            .provenance
            .data_source()
            .as_str(),
        "cache"
    );
    let missing = NnsProposalRequest::new(governance, 999);
    let report = build_nns_proposal_report_with_cache_and_source(&missing, &root, &FixtureSource)
        .await
        .unwrap();
    assert_eq!(report.provenance.data_source().as_str(), "live");
    assert_eq!(report.proposal_id, 999);

    fs::write(&path, "{}").unwrap();
    assert!(matches!(
        build_nns_proposal_list_report_with_cache_and_source(&list, &root, &FixtureSource).await,
        Err(NnsProposalHostError::Cache(
            HostCacheError::ParseCache { .. }
        ))
    ));
    assert!(matches!(
        build_nns_proposal_report_with_cache_and_source(&info, &root, &FixtureSource).await,
        Err(NnsProposalHostError::Cache(
            HostCacheError::ParseCache { .. }
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), b"{}");
    fs::remove_dir_all(root).unwrap();
}

struct FullTerminalPageSource;

impl NnsProposalSource for FullTerminalPageSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            assert_eq!(limit, 2);
            assert_eq!(before_proposal_id, None);
            assert_eq!(status, NnsProposalStatusFilter::Any);
            assert_eq!(reward_status, NnsProposalRewardStatusFilter::Any);
            Ok(NnsGovernanceSourceData::new(
                vec![
                    nns_proposal_row_from_info(proposal_info(1)),
                    nns_proposal_row_from_info(proposal_info(2)),
                ],
                fixture_provenance(request),
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        FixtureSource.fetch_proposal(request, proposal_id)
    }
}

impl NnsProposalSource for FixtureSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            assert_eq!(limit, 2);
            assert_eq!(status, NnsProposalStatusFilter::Any);
            assert_eq!(reward_status, NnsProposalRewardStatusFilter::Any);
            let rows = match before_proposal_id {
                None => vec![proposal_info(3), proposal_info(2)],
                Some(2) => vec![proposal_info(1)],
                other => panic!("unexpected before proposal id: {other:?}"),
            };
            Ok(NnsGovernanceSourceData::new(
                rows.into_iter().map(nns_proposal_row_from_info).collect(),
                fixture_provenance(request),
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        Box::pin(async move {
            Ok(NnsGovernanceSourceData::new(
                nns_proposal_row_from_info(proposal_info(proposal_id)),
                fixture_provenance(request),
            ))
        })
    }
}

fn proposal_governance_request(now_unix_secs: u64) -> NnsGovernanceRequest {
    NnsGovernanceRequest::replica_query_from_unix_secs(
        MAINNET_NETWORK,
        DEFAULT_MAINNET_ENDPOINT,
        now_unix_secs,
        "ic-query",
    )
}

fn fixture_provenance(request: &NnsGovernanceRequest) -> NnsGovernanceSourceProvenance {
    let NnsGovernanceSourceSelection::ReplicaQuery {
        endpoint,
        fetched_by,
    } = &request.source
    else {
        panic!("proposal fixture requires replica-query provenance");
    };
    NnsGovernanceSourceProvenance::ReplicaQuery {
        endpoint: endpoint.clone(),
        fetched_by: fetched_by.clone(),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_refresh_rejects_invalid_public_page_size() {
    let root = temp_dir("ic-query-nns-proposal-invalid-page-size").join("missing");
    for page_size in [0, 101] {
        let request = NnsGovernanceRefreshRequest::new(
            &root,
            MAINNET_NETWORK,
            DEFAULT_MAINNET_ENDPOINT,
            1_700_000_000,
            page_size,
        );
        let err = refresh_nns_proposal_cache_with_source_and_progress(
            &request,
            &FixtureSource,
            &mut |_| {},
        )
        .await
        .expect_err("page size must be in 1..=100");
        assert!(matches!(
            err,
            NnsProposalHostError::Proposal(NnsProposalError::InvalidLimit {
                limit,
                maximum: 100,
            }) if limit == page_size
        ));
        assert!(
            !root.exists(),
            "invalid refresh must not create cache files"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_failed_refresh_preserves_page_progress() {
    let root = temp_dir("ic-query-nns-proposal-failed-progress");
    let request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_MAINNET_ENDPOINT,
        1_700_000_000,
        2,
    )
    .with_max_pages(Some(1));

    let err =
        refresh_nns_proposal_cache_with_source_and_progress(&request, &FixtureSource, &mut |_| {})
            .await
            .expect_err("capped refresh is incomplete");

    assert!(matches!(
        err,
        NnsProposalHostError::IncompleteRefresh {
            pages_fetched: 1,
            rows_fetched: 2,
            ..
        }
    ));
    let attempt_path = nns_proposal_cache_paths(&root, MAINNET_NETWORK).refresh_attempt_path;
    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_path).expect("read failed attempt"))
            .expect("parse failed attempt");
    assert_eq!(attempt["status"], "failed");
    assert_eq!(attempt["pages_fetched"], 1);
    assert_eq!(attempt["rows_fetched"], 2);
    assert_eq!(attempt["last_cursor"], "2");
    let cache_request = NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK);
    let list = build_nns_proposal_cache_list_report(&cache_request).expect("failed attempt list");
    let status =
        build_nns_proposal_cache_status_report(&cache_request).expect("failed attempt status");
    assert_eq!(list.caches.len(), 0);
    assert_eq!(list.latest_attempt, status.latest_attempt);
    assert_eq!(
        list.latest_attempt
            .expect("failed attempt without snapshot")
            .status,
        CacheRefreshAttemptStatus::Failed
    );
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_refresh_completes_at_the_page_cap_when_full_page_reaches_id_one() {
    let root = temp_dir("ic-query-nns-proposal-full-terminal-page");
    let request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_MAINNET_ENDPOINT,
        1_700_000_000,
        2,
    )
    .with_max_pages(Some(1));
    let report = refresh_nns_proposal_cache_with_source_and_progress(
        &request,
        &FullTerminalPageSource,
        &mut |_| {},
    )
    .await
    .expect("id one proves exhaustion even with a full page");

    assert!(report.complete);
    assert_eq!(report.page_count, 1);
    assert_eq!(report.row_count, 2);
    let paths = nns_proposal_cache_paths(&root, MAINNET_NETWORK);
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&paths.snapshot_path).expect("read cache"))
            .expect("parse cache");
    assert_eq!(cache["data"]["proposals"][0]["proposal_id"], 2);
    assert_eq!(cache["data"]["proposals"][1]["proposal_id"], 1);
    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(&paths.refresh_attempt_path).expect("read attempt"))
            .expect("parse attempt");
    assert_eq!(attempt["status"], "complete");
    assert_eq!(attempt["pages_fetched"], 1);
    assert_eq!(attempt["rows_fetched"], 2);
    assert_eq!(attempt["last_cursor"], serde_json::Value::Null);
    fs::remove_dir_all(root).expect("remove fixture cache");
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_refresh_writes_complete_cache_and_status_reports() {
    let root = temp_dir("ic-query-nns-proposal-cache");
    let request = NnsGovernanceRefreshRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        cache_root: root.clone(),
        page_size: 2,
        max_pages: None,
    };

    let report =
        refresh_nns_proposal_cache_with_source_and_progress(&request, &FixtureSource, &mut |_| {})
            .await
            .expect("refresh cache");

    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(
        report.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(report.row_count, 3);
    assert_eq!(
        report.collection,
        crate::nns::NnsGovernanceCollection::Proposals
    );
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(report.page_count, 2);
    assert!(report.complete);
    assert!(report.wrote_cache);
    assert!(!report.replaced_existing_cache);

    let cache_path = nns_proposal_cache_paths(&root, MAINNET_NETWORK).snapshot_path;
    assert!(cache_path.is_file());
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(cache_path).expect("read cache")).expect("parse cache");
    assert_eq!(cache["domain"], "nns");
    let result_json = serde_json::to_value(&report).expect("serialize publication result");
    for field in [
        "network",
        "fetched_at",
        "source_endpoint",
        "fetched_by",
        "collection",
    ] {
        assert_eq!(result_json[field], cache[field], "published {field}");
    }
    assert_eq!(result_json["schema_version"], 1);
    assert_eq!(result_json["row_count"], cache["completeness"]["row_count"]);
    assert_eq!(result_json["wrote_cache"], true);
    assert_eq!(cache["entity"], "governance");
    assert_eq!(cache["collection"], "proposals");
    assert_eq!(cache["scope"], "full");

    let list = build_nns_proposal_cache_list_report(&NnsGovernanceCacheRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
    })
    .expect("cache list");

    assert_eq!(
        list.collection,
        crate::nns::NnsGovernanceCollection::Proposals
    );
    assert_eq!(list.caches.len(), 1);
    assert_eq!(list.caches[0].cache_status(), CacheValidationStatus::Valid);
    assert_eq!(list.caches[0].cache_error(), None);
    assert_eq!(list.caches[0].valid().unwrap().row_count, 3);
    assert_eq!(list.caches[0].valid().unwrap().page_count, 2);

    let status = build_nns_proposal_cache_status_report(&NnsGovernanceCacheRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root,
    })
    .expect("cache status");

    assert!(status.found);
    let summary = status.cache.as_ref().expect("validated summary");
    assert_eq!(summary.cache_status().as_str(), "ok");
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .expect("latest attempt")
            .status,
        CacheRefreshAttemptStatus::Complete
    );
    assert_eq!(list.latest_attempt.as_ref(), status.latest_attempt.as_ref());
    assert_eq!(list.refresh_attempt_path, status.refresh_attempt_path);
    assert_eq!(
        status.collection,
        crate::nns::NnsGovernanceCollection::Proposals
    );
    assert_eq!(status.cache.as_ref(), list.caches.first());
    let data = summary.valid().expect("valid cache metadata");
    assert_eq!(data.governance_canister_id, MAINNET_GOVERNANCE_CANISTER_ID);
    assert!(!data.point_in_time_guaranteed);
    let list_json = serde_json::to_value(&list).expect("list JSON");
    let status_json = serde_json::to_value(&status).expect("status JSON");
    assert_eq!(list_json["schema_version"], 1);
    assert_eq!(status_json["schema_version"], 1);
    assert_eq!(list_json["latest_attempt"], status_json["latest_attempt"]);
    assert_eq!(list_json["caches"][0], status_json["cache"]);
}

#[test]
fn nns_proposal_cache_status_reports_missing_cache() {
    let root = temp_dir("ic-query-nns-proposal-status-missing");

    let status = build_nns_proposal_cache_status_report(&NnsGovernanceCacheRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
    })
    .expect("cache status");

    assert!(!status.found);
    assert!(status.cache.is_none());
    assert!(status.latest_attempt.is_none());

    let list = build_nns_proposal_cache_list_report(&NnsGovernanceCacheRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.clone(),
    })
    .expect("cache list");
    assert_eq!(list.caches.len(), 0);
    assert_eq!(list.caches, Vec::new());

    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_cache_status_surfaces_malformed_attempt_sidecar() {
    let root = temp_dir("ic-query-nns-proposal-malformed-attempt");
    let paths = nns_proposal_cache_paths(&root, MAINNET_NETWORK);
    crate::cache_file::write_managed_text_atomically(&root, &paths.refresh_attempt_path, "{")
        .expect("write malformed attempt");
    let request = NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK);

    for result in [
        build_nns_proposal_cache_status_report(&request).map(|_| ()),
        build_nns_proposal_cache_list_report(&request).map(|_| ()),
    ] {
        assert!(matches!(
            result,
            Err(NnsProposalHostError::Cache(
                HostCacheError::ParseCache { .. }
            ))
        ));
    }

    refresh_nns_proposal_cache_with_source_and_progress(
        &NnsGovernanceRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            cache_root: root.clone(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
        &mut |_| {},
    )
    .await
    .expect("create complete snapshot");
    for invalid_snapshot in [false, true] {
        if invalid_snapshot {
            fs::write(&paths.snapshot_path, "invalid JSON").expect("corrupt snapshot");
        }
        crate::cache_file::write_managed_text_atomically(&root, &paths.refresh_attempt_path, "{")
            .expect("corrupt attempt");
        let request = NnsGovernanceCacheRequest {
            network: MAINNET_NETWORK.to_string(),
            cache_root: root.clone(),
        };
        for result in [
            build_nns_proposal_cache_status_report(&request).map(|_| ()),
            build_nns_proposal_cache_list_report(&request).map(|_| ()),
        ] {
            assert!(matches!(
                result,
                Err(NnsProposalHostError::Cache(HostCacheError::ParseCache { path, .. }))
                    if path == paths.refresh_attempt_path
            ));
        }

        fs::remove_file(&paths.refresh_attempt_path).expect("remove attempt");
        let status =
            build_nns_proposal_cache_status_report(&request).expect("absent attempt status");
        assert!(status.found);
        assert!(status.latest_attempt.is_none());
        assert_eq!(
            status
                .cache
                .as_ref()
                .expect("snapshot summary")
                .cache_error()
                .is_some(),
            invalid_snapshot
        );
        let list = build_nns_proposal_cache_list_report(&request).expect("absent attempt list");
        assert_eq!(list.caches.len(), 1);
        assert_eq!(list.caches.first(), status.cache.as_ref());
        assert_eq!(list.latest_attempt, status.latest_attempt);
    }
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_list_reads_existing_complete_cache_before_live_lookup() {
    let root = temp_dir("ic-query-nns-proposal-list-cache");
    refresh_nns_proposal_cache_with_source_and_progress(
        &NnsGovernanceRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            cache_root: root.clone(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
        &mut |_| {},
    )
    .await
    .expect("refresh cache");

    let request = NnsProposalListRequest {
        governance: proposal_governance_request(1_700_100_000),
        limit: 1,
        before_proposal_id: Some(3),
        status: NnsProposalStatusFilter::Executed,
        reward_status: NnsProposalRewardStatusFilter::Settled,
        topic: NnsProposalTopicFilter::Governance,
        proposer_neuron_id: Some(99),
        query: Some("PrOpOsAl 1".to_string()),
        sort: NnsProposalListSort::Title,
        sort_direction: SortDirection::Asc,
    };
    let report = build_nns_proposal_list_report_from_cache(&request, &root)
        .expect("cache lookup")
        .expect("cached list report");

    let json = serde_json::to_value(&report).expect("serialize cached NNS proposal list report");

    assert_eq!(
        report.context.schema_version,
        NNS_PROPOSAL_LIST_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report.provenance.data_source().as_str(), "cache");
    assert!(
        report
            .provenance
            .cache_complete()
            .expect("cache completeness")
    );
    assert_eq!(report.status_filter, NnsProposalStatus::Executed.as_str());
    assert_eq!(report.topic_filter, NnsProposalTopic::Governance.as_str());
    assert_eq!(report.proposer_filter, Some(99));
    assert_eq!(report.query_filter.as_deref(), Some("PrOpOsAl 1"));
    assert_eq!(report.sort, "title");
    assert_eq!(report.sort_direction, "asc");
    assert_eq!(report.result_scope.as_str(), "complete-cache");
    assert_eq!(report.proposals.len(), 1);
    assert_eq!(report.proposals[0].proposal_id, Some(1));
    assert_eq!(json["query_filter"], "PrOpOsAl 1");
    assert_eq!(json["result_scope"], "complete-cache");

    let mut request = request;
    request.query = None;
    request.sort = NnsProposalListSort::Id;
    request.sort_direction = SortDirection::Desc;
    let report = build_nns_proposal_list_report_from_cache(&request, &root)
        .expect("cache lookup")
        .expect("cached list report");
    assert_eq!(report.proposals[0].proposal_id, Some(2));
    assert_eq!(report.proposals.len(), 1);
    assert_eq!(report.requested_limit, 1);
    assert_eq!(report.before_proposal_id, Some(3));

    request.sort_direction = SortDirection::Asc;
    let report = build_nns_proposal_list_report_from_cache(&request, &root)
        .expect("cache lookup")
        .expect("cached list report");
    assert_eq!(report.proposals[0].proposal_id, Some(1));
    assert_eq!(report.proposals.len(), 1);
}

#[test]
fn nns_proposal_cache_reads_only_fall_back_for_missing_snapshots() {
    let root = temp_dir("ic-query-nns-proposal-cache-read-fallback").join("missing");
    let list_request = NnsProposalListRequest {
        governance: proposal_governance_request(1_700_100_000),
        limit: 25,
        before_proposal_id: None,
        status: NnsProposalStatusFilter::Any,
        reward_status: NnsProposalRewardStatusFilter::Any,
        topic: NnsProposalTopicFilter::Any,
        proposer_neuron_id: None,
        query: None,
        sort: NnsProposalListSort::Api,
        sort_direction: SortDirection::Desc,
    };
    let detail_request = NnsProposalRequest {
        governance: list_request.governance.clone(),
        proposal_id: 42,
    };
    let read_list = || build_nns_proposal_list_report_from_cache(&list_request, &root);
    let read_detail = || build_nns_proposal_report_from_cache(&detail_request, &root);

    assert!(read_list().expect("missing list cache").is_none());
    assert!(read_detail().expect("missing detail cache").is_none());
    assert!(!root.exists(), "cache-only reads do not create files");

    let path = nns_proposal_cache_paths(&root, MAINNET_NETWORK).snapshot_path;
    crate::cache_file::write_managed_text_atomically(&root, &path, "{")
        .expect("write invalid cache");
    for error in [
        read_list().expect_err("invalid list cache"),
        read_detail().expect_err("invalid detail cache"),
    ] {
        assert!(matches!(
            error,
            NnsProposalHostError::Cache(HostCacheError::ParseCache { path: actual, .. })
                if actual == path
        ));
    }
    #[cfg(unix)]
    {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o664))
            .expect("widen file permissions");
        for error in [
            read_list().expect_err("unsafe list cache"),
            read_detail().expect_err("unsafe detail cache"),
        ] {
            assert!(matches!(
                error,
                NnsProposalHostError::Cache(HostCacheError::Operation {
                    source: crate::CacheFileError::UnsafeManagedPermissions { .. },
                    ..
                })
            ));
        }
    }
    fs::remove_dir_all(root).expect("remove cache fixture");
}

#[tokio::test(flavor = "current_thread")]
async fn cached_proposal_activity_preserves_collection_evidence_and_window_boundaries() {
    let root = temp_dir("ic-query-nns-cached-activity").join("missing");
    let cache_request = NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK);
    assert!(matches!(
        build_nns_proposal_activity_report_from_cache(
            &cache_request,
            &NnsProposalActivityRequest::default()
        ),
        Err(NnsProposalHostError::MissingProposalCache { .. })
    ));
    assert!(!root.exists());
    let path = refresh_fixture_nns_proposal_cache(&root).await;
    let original = fs::read(&path).expect("read snapshot");
    let report = build_nns_proposal_activity_report_from_cache(
        &cache_request,
        &NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: Some(1_700_000_002),
            until_proposal_timestamp_seconds: Some(1_700_000_003),
        },
    )
    .expect("cached activity");
    let cache: super::model::NnsProposalCache = serde_json::from_slice(&original).unwrap();
    let raw_report = crate::nns::proposals::build_nns_proposal_activity_report(
        &NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: Some(1_700_000_002),
            until_proposal_timestamp_seconds: Some(1_700_000_003),
        },
        &crate::nns::proposals::NnsProposalCollectionState::from_complete_collection(
            cache.data.collection_evidence.restore_state(&cache),
            cache.completeness.row_count as u64,
        ),
        &cache.data.proposals,
    )
    .unwrap();
    assert_eq!(report, raw_report);
    assert_eq!(report.collected_proposal_count, 3);
    assert_eq!(report.included_proposal_count, 1);
    assert_eq!(report.excluded_before_from_count, 1);
    assert_eq!(report.excluded_at_or_after_until_count, 1);
    assert_eq!(report.context.collection_page_count, 2);
    assert_eq!(report.context.collection_started_at, "2023-11-14T22:13:20Z");
    assert_eq!(report.context.collection_updated_at, "2023-11-14T22:13:20Z");
    assert_eq!(
        report.context.source,
        fixture_provenance(&proposal_governance_request(1_700_000_000))
    );
    assert!(!report.context.point_in_time_guaranteed);
    assert_eq!(fs::read(&path).expect("read unchanged snapshot"), original);
    assert!(matches!(
        build_nns_proposal_activity_report_from_cache(
            &cache_request,
            &NnsProposalActivityRequest {
                from_proposal_timestamp_seconds: Some(10),
                until_proposal_timestamp_seconds: Some(10),
            }
        ),
        Err(NnsProposalHostError::Activity(
            NnsProposalActivityError::InvalidTimeWindow { .. }
        ))
    ));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[tokio::test(flavor = "current_thread")]
async fn cached_proposal_activity_rejects_inconsistent_collection_evidence() {
    let root = temp_dir("ic-query-nns-cached-activity-invalid");
    let path = refresh_fixture_nns_proposal_cache(&root).await;
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let request = NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK);
    for (field, value) in [
        ("started_at", serde_json::json!("2023-11-14T22:14:20Z")),
        ("started_at", serde_json::json!("invalid timestamp")),
        ("max_pages", serde_json::json!(1)),
        (
            "requested_source",
            serde_json::json!({"source_transport": "replica_query", "endpoint": "https://example.com", "fetched_by": "ic-query"}),
        ),
    ] {
        let mut cache = original.clone();
        cache["data"]["collection_evidence"][field] = value;
        fs::write(&path, serde_json::to_vec(&cache).unwrap()).unwrap();
        assert!(
            matches!(
                build_nns_proposal_activity_report_from_cache(
                    &request,
                    &NnsProposalActivityRequest::default()
                ),
                Err(NnsProposalHostError::InvalidCache { .. })
            ),
            "field {field}"
        );
    }
    fs::remove_dir_all(root).expect("remove fixture");
}

#[tokio::test(flavor = "current_thread")]
async fn cached_proposal_activity_checks_timestamps_before_view_filtering() {
    let root = temp_dir("ic-query-nns-cached-activity-timestamps");
    let path = refresh_fixture_nns_proposal_cache(&root).await;
    let mut cache: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let proposal_id = cache["data"]["proposals"][0]["proposal_id"]
        .as_u64()
        .unwrap();
    cache["data"]["proposals"][0]["proposal_timestamp_seconds"] = serde_json::json!(0);
    let bytes = serde_json::to_vec(&cache).unwrap();
    fs::write(&path, &bytes).unwrap();
    let error = build_nns_proposal_activity_report_from_cache(
        &NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK),
        &NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: Some(1),
            until_proposal_timestamp_seconds: None,
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        NnsProposalHostError::Activity(NnsProposalActivityError::ZeroProposalTimestamp {
            proposal_id: actual,
        }) if actual == proposal_id
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_cache_rejects_missing_zero_and_duplicate_ids() {
    let root = temp_dir("ic-query-nns-proposal-invalid-ids");
    let path = refresh_fixture_nns_proposal_cache(&root).await;
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("read cache")).expect("parse cache");
    let list_request = NnsProposalListRequest::new(proposal_governance_request(1_700_100_000), 25);
    let detail_request = NnsProposalRequest::new(list_request.governance.clone(), 1);

    for (id, reason) in [
        (serde_json::Value::Null, "without an id"),
        (serde_json::json!(0), "id zero"),
        (
            original["data"]["proposals"][1]["proposal_id"].clone(),
            "duplicate proposal id",
        ),
    ] {
        let mut cache = original.clone();
        cache["data"]["proposals"][0]["proposal_id"] = id;
        fs::write(
            &path,
            serde_json::to_vec_pretty(&cache).expect("serialize cache"),
        )
        .expect("write invalid cache");

        for error in [
            build_nns_proposal_list_report_from_cache(&list_request, &root)
                .expect_err("invalid list cache"),
            build_nns_proposal_report_from_cache(&detail_request, &root)
                .expect_err("invalid detail cache"),
            build_nns_proposal_activity_report_from_cache(
                &NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK),
                &NnsProposalActivityRequest::default(),
            )
            .expect_err("invalid activity cache"),
        ] {
            assert!(matches!(
                error,
                NnsProposalHostError::InvalidCache { path: actual, reason: actual_reason }
                    if actual == path && actual_reason.contains(reason)
            ));
        }
        assert_invalid_nns_proposal_cache_status(&root, reason);
    }
    fs::remove_dir_all(root).expect("remove cache fixture");
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_cache_status_reports_snapshot_identity_mismatch() {
    let root = temp_dir("ic-query-nns-proposal-identity-mismatch");
    let cache_path = refresh_fixture_nns_proposal_cache(&root).await;
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

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_cache_status_reports_unsupported_schema() {
    let root = temp_dir("ic-query-nns-proposal-unsupported-schema");
    let cache_path = refresh_fixture_nns_proposal_cache(&root).await;
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

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_cache_rejects_false_point_in_time_authority() {
    let root = temp_dir("ic-query-nns-proposal-point-in-time");
    let cache_path = refresh_fixture_nns_proposal_cache(&root).await;
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache")).expect("parse cache");
    cache["completeness"]["point_in_time_guaranteed"] = serde_json::json!(true);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize cache"),
    )
    .expect("write cache");

    assert_invalid_nns_proposal_cache_status(&root, "point-in-time guarantee");

    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_cache_status_reports_malformed_json() {
    let root = temp_dir("ic-query-nns-proposal-malformed-json");
    let cache_path = refresh_fixture_nns_proposal_cache(&root).await;
    fs::write(&cache_path, "{").expect("write malformed cache");

    assert_invalid_nns_proposal_cache_status(&root, "failed to parse NNS proposal cache");

    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_detail_reads_existing_complete_cache_before_live_lookup() {
    let root = temp_dir("ic-query-nns-proposal-detail-cache");
    refresh_nns_proposal_cache_with_source_and_progress(
        &NnsGovernanceRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            cache_root: root.clone(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
        &mut |_| {},
    )
    .await
    .expect("refresh cache");

    let request = NnsProposalRequest {
        governance: proposal_governance_request(1_700_100_000),
        proposal_id: 2,
    };
    let report = build_nns_proposal_report_from_cache(&request, &root)
        .expect("cache lookup")
        .expect("cached proposal report");

    assert_eq!(
        report.context.schema_version,
        NNS_PROPOSAL_REPORT_SCHEMA_VERSION
    );
    assert_eq!(report.proposal_id, 2);
    assert_eq!(report.proposal.title.as_deref(), Some("Proposal 2"));
    assert_eq!(report.provenance.data_source().as_str(), "cache");
    assert!(
        report
            .provenance
            .cache_complete()
            .expect("cache completeness")
    );
    assert!(
        report
            .provenance
            .cache_path()
            .expect("cache path")
            .ends_with("nns/ic/governance/proposals/full.json")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn nns_proposal_detail_cache_lookup_returns_none_for_missing_cached_proposal() {
    let root = temp_dir("ic-query-nns-proposal-detail-cache-missing");
    refresh_nns_proposal_cache_with_source_and_progress(
        &NnsGovernanceRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            cache_root: root.clone(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
        &mut |_| {},
    )
    .await
    .expect("refresh cache");

    let report = build_nns_proposal_report_from_cache(
        &NnsProposalRequest {
            governance: proposal_governance_request(1_700_100_000),
            proposal_id: 42,
        },
        &root,
    )
    .expect("cache lookup");

    assert!(report.is_none());
}

async fn refresh_fixture_nns_proposal_cache(root: &std::path::Path) -> std::path::PathBuf {
    refresh_nns_proposal_cache_with_source_and_progress(
        &NnsGovernanceRefreshRequest {
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            now_unix_secs: 1_700_000_000,
            cache_root: root.to_path_buf(),
            page_size: 2,
            max_pages: None,
        },
        &FixtureSource,
        &mut |_| {},
    )
    .await
    .expect("refresh cache");
    nns_proposal_cache_paths(root, MAINNET_NETWORK).snapshot_path
}

fn assert_invalid_nns_proposal_cache_status(root: &std::path::Path, expected_error: &str) {
    let status = build_nns_proposal_cache_status_report(&NnsGovernanceCacheRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.to_path_buf(),
    })
    .expect("cache status");

    let cache = status.cache.as_ref().expect("cache summary");

    assert!(status.found);
    assert_eq!(
        status.collection,
        crate::nns::NnsGovernanceCollection::Proposals
    );
    assert_eq!(cache.cache_status(), CacheValidationStatus::Invalid);
    assert!(cache.valid().is_none());
    let invalid = serde_json::to_value(cache).unwrap();
    for field in [
        "governance_canister_id",
        "complete",
        "point_in_time_guaranteed",
        "row_count",
        "page_count",
        "page_size",
        "fetched_at",
        "source_endpoint",
    ] {
        assert!(
            invalid.get(field).is_none(),
            "invalid summary has no {field}"
        );
    }
    assert!(
        cache
            .cache_error()
            .is_some_and(|error| error.contains(expected_error))
    );

    let list = build_nns_proposal_cache_list_report(&NnsGovernanceCacheRequest {
        network: MAINNET_NETWORK.to_string(),
        cache_root: root.to_path_buf(),
    })
    .expect("cache list");
    assert_eq!(list.caches.len(), 1);
    assert_eq!(
        list.caches[0].cache_status(),
        CacheValidationStatus::Invalid
    );
    assert!(
        list.caches[0]
            .cache_error()
            .is_some_and(|error| error.contains(expected_error))
    );
}

fn proposal_info(proposal_id: u64) -> NnsProposalInfo {
    NnsProposalInfo {
        id: Some(NnsGovernanceProposalId { id: proposal_id }),
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
        proposer: Some(NnsGovernanceNeuronId { id: 99 }),
        executed_timestamp_seconds: 1_700_000_300,
        total_potential_voting_power: Some(100),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn proposal_cache_preserves_unknown_votes_and_refuses_unknown_fields() {
    let root = temp_dir("ic-query-proposal-ballot-evidence");
    let path = refresh_fixture_nns_proposal_cache(&root).await;
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let request = NnsProposalListRequest::new(proposal_governance_request(1_700_100_000), 25);
    for code in [-9, 99] {
        let mut cache = original.clone();
        cache["data"]["proposals"][0]["ballots"][0]["vote"] = serde_json::json!(code);
        fs::write(&path, serde_json::to_vec_pretty(&cache).unwrap()).unwrap();
        let report = build_nns_proposal_list_report_from_cache(&request, &root)
            .unwrap()
            .unwrap();
        let ballot = &report.proposals[0].ballots[0];
        assert_eq!(ballot.vote, code);
        cache["data"]["proposals"][0]["ballots"][0]["unexpected_evidence"] =
            serde_json::json!(true);
        let bytes = serde_json::to_vec_pretty(&cache).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(
            matches!(build_nns_proposal_list_report_from_cache(&request, &root), Err(NnsProposalHostError::Cache(crate::HostCacheError::ParseCache { path: actual, .. })) if actual == path)
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    refresh_fixture_nns_proposal_cache(&root).await;
    build_nns_proposal_list_report_from_cache(&request, &root)
        .expect("explicit refresh restores valid cache")
        .expect("refreshed complete snapshot");
    fs::remove_dir_all(root).unwrap();
}
