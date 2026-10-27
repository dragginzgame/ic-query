use super::{
    DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, NnsKnownNeuronData, NnsNeuronError, NnsNeuronHostError,
    NnsNeuronInfoRequest, NnsNeuronListRequest, NnsNeuronPage, NnsNeuronRow, NnsNeuronSource,
    NnsNeuronSourceFuture, NnsNeuronVisibility, build_nns_neuron_cache_status_report,
    build_nns_neuron_distribution_report_from_cache, build_nns_neuron_info_report_from_cache,
    build_nns_neuron_info_report_with_cache_and_source, build_nns_neuron_info_report_with_source,
    build_nns_neuron_list_report_from_cache, build_nns_neuron_list_report_with_cache_and_source,
    build_nns_neuron_list_report_with_source, nns_neuron_cache_path,
    nns_neuron_refresh_attempt_path, refresh_nns_neuron_cache_with_source,
};
use crate::{
    cache::{CacheRefreshAttemptStatus, CacheValidationStatus},
    nns::{
        LiveNnsSource, MAINNET_GOVERNANCE_CANISTER_ID, NnsGovernanceCacheRequest,
        NnsGovernanceRefreshRequest,
        governance::{
            NnsGovernanceError, NnsGovernanceRequest, NnsGovernanceSourceData,
            NnsGovernanceSourceProvenance, NnsGovernanceSourceSelection,
        },
    },
    subnet_catalog::MAINNET_NETWORK,
    test_support::temp_dir,
};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

static LIVE_SOURCE_CALLS: AtomicUsize = AtomicUsize::new(0);

struct FixtureSource;

#[tokio::test(flavor = "current_thread")]
async fn cache_aware_neuron_builders_preserve_fallback_and_strict_cache_errors() {
    let root = temp_dir("ic-query-nns-neuron-cache-source");
    let governance = governance_request(1_700_000_001);
    let list = NnsNeuronListRequest::new(governance.clone(), 2);
    let info = NnsNeuronInfoRequest::new(governance.clone(), 2);
    assert!(
        !build_nns_neuron_list_report_with_cache_and_source(&list, &root, &FixtureSource)
            .await
            .unwrap()
            .from_cache
    );
    assert!(
        !build_nns_neuron_info_report_with_cache_and_source(&info, &root, &FixtureSource)
            .await
            .unwrap()
            .from_cache
    );

    refresh_nns_neuron_cache_with_source(
        &NnsGovernanceRefreshRequest::new(
            &root,
            MAINNET_NETWORK,
            DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
            1_700_000_000,
            2,
        ),
        &FixtureSource,
    )
    .await
    .unwrap();
    assert!(
        build_nns_neuron_list_report_with_cache_and_source(&list, &root, &FixtureSource)
            .await
            .unwrap()
            .from_cache
    );
    assert!(
        build_nns_neuron_info_report_with_cache_and_source(&info, &root, &FixtureSource)
            .await
            .unwrap()
            .from_cache
    );
    let missing = NnsNeuronInfoRequest::new(governance, 999);
    let report =
        build_nns_neuron_info_report_with_cache_and_source(&missing, &root, &FixtureSource)
            .await
            .unwrap();
    assert!(!report.from_cache);
    assert_eq!(report.neuron.neuron_id, 999);

    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    fs::write(&path, "{}").unwrap();
    assert!(matches!(
        build_nns_neuron_list_report_with_cache_and_source(&list, &root, &FixtureSource).await,
        Err(NnsNeuronHostError::Cache(
            crate::HostCacheError::ParseCache { .. }
        ))
    ));
    assert!(matches!(
        build_nns_neuron_info_report_with_cache_and_source(&info, &root, &FixtureSource).await,
        Err(NnsNeuronHostError::Cache(
            crate::HostCacheError::ParseCache { .. }
        ))
    ));
    assert_eq!(fs::read(&path).unwrap(), b"{}");
    fs::remove_dir_all(root).unwrap();
}

impl NnsNeuronSource for FixtureSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            assert_eq!(request.network, MAINNET_NETWORK);
            assert_replica_request(request, DEFAULT_NNS_NEURON_SOURCE_ENDPOINT);
            assert_eq!(page_size, 2);
            let (neurons, next_start_neuron_id) = match exclusive_start_neuron_id {
                None => (vec![sample_neuron(1), sample_neuron(2)], Some(2)),
                Some(2) => (vec![sample_neuron(3)], None),
                other => panic!("unexpected neuron cursor: {other:?}"),
            };
            Ok(fixture_source_data(
                request,
                NnsNeuronPage {
                    neurons,
                    next_start_neuron_id,
                },
            ))
        })
    }

    fn fetch_neuron<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        Box::pin(async move {
            assert_eq!(request.network, MAINNET_NETWORK);
            Ok(fixture_source_data(request, sample_neuron(neuron_id)))
        })
    }
}

struct CountingSource;

struct MutatingLastPageSource(fn(&mut NnsNeuronPage));

impl NnsNeuronSource for MutatingLastPageSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            let mut data = FixtureSource
                .fetch_neuron_page(request, exclusive_start_neuron_id, page_size)
                .await?;
            if exclusive_start_neuron_id.is_some() {
                self.0(&mut data.value);
            }
            Ok(data)
        })
    }

    fn fetch_neuron<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        FixtureSource.fetch_neuron(request, neuron_id)
    }
}

impl NnsNeuronSource for CountingSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _exclusive_start_neuron_id: Option<u64>,
        _page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        LIVE_SOURCE_CALLS.fetch_add(1, Ordering::SeqCst);
        unreachable!("unsupported network must be rejected before source invocation")
    }

    fn fetch_neuron<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        LIVE_SOURCE_CALLS.fetch_add(1, Ordering::SeqCst);
        unreachable!("unsupported network must be rejected before source invocation")
    }
}

fn governance_request(now_unix_secs: u64) -> NnsGovernanceRequest {
    NnsGovernanceRequest::replica_query_from_unix_secs(
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        now_unix_secs,
        "ic-query",
    )
}

fn fixture_source_data<T>(request: &NnsGovernanceRequest, value: T) -> NnsGovernanceSourceData<T> {
    let NnsGovernanceSourceSelection::ReplicaQuery {
        endpoint,
        fetched_by,
    } = &request.source
    else {
        panic!("fixture requires a replica query request")
    };
    NnsGovernanceSourceData::new(
        value,
        NnsGovernanceSourceProvenance::ReplicaQuery {
            endpoint: endpoint.clone(),
            fetched_by: fetched_by.clone(),
        },
    )
}

fn assert_replica_request(request: &NnsGovernanceRequest, expected_endpoint: &str) {
    let NnsGovernanceSourceSelection::ReplicaQuery {
        endpoint,
        fetched_by,
    } = &request.source
    else {
        panic!("expected replica query request")
    };
    assert_eq!(endpoint, expected_endpoint);
    assert_eq!(fetched_by, "ic-query");
}

#[tokio::test(flavor = "current_thread")]
async fn cached_neuron_distribution_preserves_collection_evidence_and_raw_totals() {
    let root = temp_dir("ic-query-nns-cached-distribution").join("missing");
    let request = NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK);
    assert!(matches!(
        build_nns_neuron_distribution_report_from_cache(&request),
        Err(NnsNeuronHostError::MissingNeuronCache { .. })
    ));
    assert!(!root.exists());
    refresh_nns_neuron_cache_with_source(
        &NnsGovernanceRefreshRequest::new(
            &root,
            MAINNET_NETWORK,
            DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
            1_700_000_000,
            2,
        ),
        &FixtureSource,
    )
    .await
    .expect("refresh fixture");
    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    let original = fs::read(&path).unwrap();
    let report =
        build_nns_neuron_distribution_report_from_cache(&request).expect("cached distribution");
    assert_eq!(report.collected_neuron_count, 3);
    assert_eq!(report.total_effective_stake_e8s, 600_000_000);
    assert_eq!(report.collection_page_count, 2);
    assert_eq!(report.collection_started_at, "2023-11-14T22:13:20Z");
    assert_eq!(report.collection_updated_at, "2023-11-14T22:13:20Z");
    assert_eq!(
        report.source,
        fixture_source_data(&governance_request(1_700_000_000), ()).provenance
    );
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_dir_all(root).expect("remove fixture");
}

#[tokio::test(flavor = "current_thread")]
async fn cached_neuron_distribution_rejects_inconsistent_collection_evidence() {
    let root = temp_dir("ic-query-nns-cached-distribution-invalid");
    refresh_nns_neuron_cache_with_source(
        &NnsGovernanceRefreshRequest::new(
            &root,
            MAINNET_NETWORK,
            DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
            1_700_000_000,
            2,
        ),
        &FixtureSource,
    )
    .await
    .expect("refresh fixture");
    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let request = NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK);
    for (field, value) in [
        ("network", serde_json::json!("invalid")),
        ("updated_at", serde_json::json!("2023-11-14T22:14:20Z")),
        ("neurons_fetched", serde_json::json!(2)),
        ("status", serde_json::json!("collecting")),
        (
            "source",
            serde_json::json!({"source_transport": "replica_query", "endpoint": "https://example.com", "fetched_by": "ic-query"}),
        ),
    ] {
        let mut cache = original.clone();
        cache["data"]["collection_state"][field] = value;
        fs::write(&path, serde_json::to_vec(&cache).unwrap()).unwrap();
        assert!(
            matches!(
                build_nns_neuron_distribution_report_from_cache(&request),
                Err(NnsNeuronHostError::InvalidCache { .. })
            ),
            "field {field}"
        );
    }
    fs::remove_dir_all(root).expect("remove fixture");
}

#[tokio::test(flavor = "current_thread")]
async fn public_list_and_info_reports_preserve_governance_rows() {
    let list_request = NnsNeuronListRequest::new(governance_request(1_700_000_000), 2);
    let list = build_nns_neuron_list_report_with_source(&list_request, &FixtureSource)
        .await
        .expect("live list report");

    assert!(!list.from_cache);
    assert_eq!(list.cache_path, None);
    assert_eq!(list.next_start_neuron_id, Some(2));
    assert_eq!(
        list.neurons
            .iter()
            .map(|row| row.neuron_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(list.neurons[0].visibility, Some(2));
    assert_eq!(
        NnsNeuronVisibility::from_code(list.neurons[0].visibility),
        NnsNeuronVisibility::Public
    );

    let info_request = NnsNeuronInfoRequest::new(governance_request(1_700_000_000), 42);
    let info = build_nns_neuron_info_report_with_source(&info_request, &FixtureSource)
        .await
        .expect("live info report");

    assert!(!info.from_cache);
    assert_eq!(info.cache_path, None);
    assert_eq!(info.neuron.neuron_id, 42);
    assert_eq!(
        info.neuron
            .known_neuron_data
            .as_ref()
            .expect("known neuron")
            .name,
        "Neuron 42"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn public_builders_reject_non_mainnet_before_invoking_a_source() {
    LIVE_SOURCE_CALLS.store(0, Ordering::SeqCst);
    let list_request = NnsNeuronListRequest::new(
        NnsGovernanceRequest::replica_query_from_unix_secs(
            "local",
            "http://127.0.0.1:1",
            1_700_000_000,
            "test",
        ),
        2,
    );
    let list_error = build_nns_neuron_list_report_with_source(&list_request, &CountingSource)
        .await
        .expect_err("non-mainnet list must fail");
    assert!(matches!(
        list_error,
        NnsNeuronError::Governance(NnsGovernanceError::UnsupportedNetwork { ref network })
            if network == "local"
    ));

    let info_request = NnsNeuronInfoRequest::new(
        NnsGovernanceRequest::replica_query_from_unix_secs(
            "local",
            "http://127.0.0.1:1",
            1_700_000_000,
            "test",
        ),
        1,
    );
    let info_error = build_nns_neuron_info_report_with_source(&info_request, &CountingSource)
        .await
        .expect_err("non-mainnet info must fail");
    assert!(matches!(
        info_error,
        NnsNeuronError::Governance(NnsGovernanceError::UnsupportedNetwork { ref network })
            if network == "local"
    ));
    assert_eq!(LIVE_SOURCE_CALLS.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn live_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsGovernanceRequest::replica_query(
        "local",
        "not a valid replica endpoint",
        "2023-11-14T22:13:20Z",
        "test",
    );

    let error = LiveNnsSource::default()
        .fetch_neuron_page(&request, None, 2)
        .await
        .expect_err("live source must reject non-mainnet");

    assert!(matches!(
        error,
        NnsNeuronError::Governance(NnsGovernanceError::UnsupportedNetwork { ref network })
            if network == "local"
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn list_rejects_a_cursor_that_does_not_match_the_page() {
    struct InvalidCursorSource;

    impl NnsNeuronSource for InvalidCursorSource {
        fn fetch_neuron_page<'a>(
            &'a self,
            request: &'a NnsGovernanceRequest,
            _exclusive_start_neuron_id: Option<u64>,
            _page_size: u32,
        ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
            Box::pin(async move {
                Ok(fixture_source_data(
                    request,
                    NnsNeuronPage {
                        neurons: vec![sample_neuron(1), sample_neuron(2)],
                        next_start_neuron_id: Some(1),
                    },
                ))
            })
        }

        fn fetch_neuron<'a>(
            &'a self,
            request: &'a NnsGovernanceRequest,
            neuron_id: u64,
        ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
            Box::pin(async move { Ok(fixture_source_data(request, sample_neuron(neuron_id))) })
        }
    }

    let request = NnsNeuronListRequest::new(governance_request(1_700_000_000), 2);
    let error = build_nns_neuron_list_report_with_source(&request, &InvalidCursorSource)
        .await
        .expect_err("invalid cursor must fail");

    assert!(matches!(error, NnsNeuronError::InvalidResponse { .. }));
}

#[tokio::test(flavor = "current_thread")]
async fn refresh_publishes_one_complete_snapshot_for_cached_list_and_info() {
    let root = temp_dir("ic-query-nns-neuron-cache");
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );

    let refresh = refresh_nns_neuron_cache_with_source(&refresh_request, &FixtureSource)
        .await
        .expect("refresh complete neuron cache");

    assert_eq!(refresh.row_count, 3);
    assert_eq!(refresh.collection, "neurons");
    assert!(refresh.wrote_cache);
    assert_eq!(refresh.page_count, 2);
    assert!(refresh.complete);
    assert!(!refresh.point_in_time_guaranteed);
    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    assert!(path.is_file());
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("read neuron cache"))
            .expect("parse neuron cache");
    assert_eq!(cache["collection"], "neurons");
    let result_json = serde_json::to_value(&refresh).expect("serialize publication result");
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
    assert_eq!(result_json["point_in_time_guaranteed"], false);
    assert_eq!(cache["completeness"]["status"], "api_exhausted");
    assert_eq!(cache["completeness"]["point_in_time_guaranteed"], false);

    let list_request = NnsNeuronListRequest::new(governance_request(1_700_000_001), 2)
        .with_exclusive_start_neuron_id(1);
    let list = build_nns_neuron_list_report_from_cache(&list_request, &root)
        .expect("load cached list")
        .expect("complete cache");
    assert!(list.from_cache);
    assert_eq!(list.cache_path, Some(path.display().to_string()));
    assert_eq!(list.total_neuron_count, Some(3));
    assert!(!list.point_in_time_guaranteed);
    assert_eq!(
        list.neurons
            .iter()
            .map(|row| row.neuron_id)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(list.next_start_neuron_id, None);

    let info_request = NnsNeuronInfoRequest::new(governance_request(1_700_000_001), 2);
    let info = build_nns_neuron_info_report_from_cache(&info_request, &root)
        .expect("load cached detail")
        .expect("cached neuron");
    assert!(info.from_cache);
    assert_eq!(info.cache_path, Some(path.display().to_string()));
    assert_eq!(info.neuron.neuron_id, 2);

    let status = build_nns_neuron_cache_status_report(&NnsGovernanceCacheRequest::new(
        &root,
        MAINNET_NETWORK,
    ))
    .expect("cache status");
    assert!(status.found);
    assert_eq!(status.collection, "neurons");
    let summary = status.cache.as_ref().expect("cache summary");
    assert_eq!(
        summary.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert!(summary.complete);
    assert!(!summary.point_in_time_guaranteed);
    assert_eq!(summary.row_count, 3);
    assert_eq!(
        status.cache.as_ref().expect("cache summary").cache_status,
        CacheValidationStatus::Valid
    );
    assert_eq!(
        status.latest_attempt.as_ref().expect("attempt").status,
        CacheRefreshAttemptStatus::Complete
    );
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn cached_neuron_reports_return_typed_snapshot_identity_mismatches() {
    let root = temp_dir("ic-query-nns-neuron-cache-identity");
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    refresh_nns_neuron_cache_with_source(&refresh_request, &FixtureSource)
        .await
        .expect("refresh complete neuron cache");
    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("read neuron cache"))
            .expect("parse neuron cache");
    cache["collection"] = serde_json::json!("wrong");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&cache).expect("serialize invalid cache"),
    )
    .expect("replace neuron cache");

    let request = NnsNeuronListRequest::new(governance_request(1_700_000_001), 2);
    let error = build_nns_neuron_list_report_from_cache(&request, &root)
        .expect_err("identity mismatch must remain typed");

    assert!(matches!(
        error,
        NnsNeuronHostError::CacheIdentityMismatch {
            path: error_path,
            field: "collection",
            expected,
            actual,
        } if error_path == path && expected == "neurons" && actual == "wrong"
    ));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn refresh_rejects_invalid_later_pages_and_preserves_complete_snapshot() {
    let root = temp_dir("ic-query-nns-neuron-invalid-page");
    let request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    refresh_nns_neuron_cache_with_source(&request, &FixtureSource)
        .await
        .expect("publish initial complete cache");
    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    let original = fs::read(&path).expect("read initial snapshot");

    for mutate in [
        (|page: &mut NnsNeuronPage| page.neurons[0].neuron_id = 2) as fn(&mut NnsNeuronPage),
        |page| page.next_start_neuron_id = Some(3),
    ] {
        let error = refresh_nns_neuron_cache_with_source(&request, &MutatingLastPageSource(mutate))
            .await
            .expect_err("invalid page must stop refresh");
        assert!(matches!(
            error,
            NnsNeuronHostError::Neuron(NnsNeuronError::InvalidResponse { .. })
        ));
        assert_eq!(fs::read(&path).expect("preserved snapshot"), original);
        let distribution = build_nns_neuron_distribution_report_from_cache(
            &NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK),
        )
        .expect("last complete distribution survives failed refresh");
        assert_eq!(distribution.collected_neuron_count, 3);

        let status = build_nns_neuron_cache_status_report(&NnsGovernanceCacheRequest::new(
            &root,
            MAINNET_NETWORK,
        ))
        .expect("cache status after rejected page");
        assert_eq!(
            status.cache.expect("complete cache").cache_status,
            CacheValidationStatus::Valid
        );
        let attempt = status.latest_attempt.expect("failed attempt");
        assert_eq!(attempt.status, CacheRefreshAttemptStatus::Failed);
        assert_eq!(attempt.pages_fetched, 1);
        assert_eq!(attempt.rows_fetched, 2);
        assert_eq!(attempt.last_cursor.as_deref(), Some("2"));
    }
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn refresh_completes_at_the_page_cap_after_an_empty_terminal_neuron_page() {
    let root = temp_dir("ic-query-nns-neuron-empty-terminal-page");
    let request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    )
    .with_max_pages(Some(2));
    let source = MutatingLastPageSource(|page| page.neurons.clear());
    let report = refresh_nns_neuron_cache_with_source(&request, &source)
        .await
        .expect("empty terminal page proves exhaustion at the page cap");

    assert!(report.complete);
    assert_eq!(report.page_count, 2);
    assert_eq!(report.row_count, 2);
    let status = build_nns_neuron_cache_status_report(&NnsGovernanceCacheRequest::new(
        &root,
        MAINNET_NETWORK,
    ))
    .expect("published cache status");
    assert_eq!(
        status.cache.expect("complete cache").cache_status,
        CacheValidationStatus::Valid
    );
    let attempt = status.latest_attempt.expect("complete attempt");
    assert_eq!(attempt.status, CacheRefreshAttemptStatus::Complete);
    assert_eq!(attempt.pages_fetched, 2);
    assert_eq!(attempt.rows_fetched, 2);
    assert_eq!(attempt.last_cursor, None);
    fs::remove_dir_all(root).expect("remove fixture cache");
}

#[tokio::test(flavor = "current_thread")]
async fn capped_refresh_keeps_failure_evidence_without_publishing_a_snapshot() {
    let root = temp_dir("ic-query-nns-neuron-incomplete");
    let request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    )
    .with_max_pages(Some(1));

    let error = refresh_nns_neuron_cache_with_source(&request, &FixtureSource)
        .await
        .expect_err("capped refresh must remain incomplete");

    assert!(matches!(
        error,
        NnsNeuronHostError::IncompleteRefresh {
            pages_fetched: 1,
            rows_fetched: 2,
            ..
        }
    ));
    assert!(!nns_neuron_cache_path(&root, MAINNET_NETWORK).exists());
    let status = build_nns_neuron_cache_status_report(&NnsGovernanceCacheRequest::new(
        &root,
        MAINNET_NETWORK,
    ))
    .expect("cache status");
    assert!(!status.found);
    let attempt = status.latest_attempt.expect("failed attempt");
    assert_eq!(attempt.status, CacheRefreshAttemptStatus::Failed);
    assert_eq!(attempt.pages_fetched, 1);
    assert_eq!(attempt.rows_fetched, 2);
    assert_eq!(attempt.last_cursor.as_deref(), Some("2"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn cache_status_distinguishes_invalid_attempt_evidence_from_the_snapshot() {
    let root = temp_dir("ic-query-nns-neuron-invalid-attempt");
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    refresh_nns_neuron_cache_with_source(&refresh_request, &FixtureSource)
        .await
        .expect("refresh complete neuron cache");
    let attempt_path = nns_neuron_refresh_attempt_path(&root, MAINNET_NETWORK);
    let mut attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("read attempt"))
            .expect("parse attempt");
    attempt["metadata"]["governance_canister_id"] =
        serde_json::Value::String("aaaaa-aa".to_string());
    let invalid_attempt = serde_json::to_string_pretty(&attempt).expect("serialize attempt");
    crate::cache_file::write_managed_text_atomically(&root, &attempt_path, &invalid_attempt)
        .expect("replace attempt");

    let error = build_nns_neuron_cache_status_report(&NnsGovernanceCacheRequest::new(
        &root,
        MAINNET_NETWORK,
    ))
    .expect_err("invalid attempt must remain distinct from the complete snapshot");

    assert!(matches!(
        error,
        NnsNeuronHostError::InvalidRefreshAttempt { path, reason }
            if path == attempt_path && reason.contains("governance_canister_id")
    ));
    let _ = fs::remove_dir_all(root);
}

fn sample_neuron(neuron_id: u64) -> NnsNeuronRow {
    NnsNeuronRow {
        neuron_id,
        state: 1,
        visibility: Some(2),

        neuron_type: None,
        stake_e8s: neuron_id.saturating_mul(100_000_000),
        staked_maturity_e8s_equivalent: Some(10),
        dissolve_delay_seconds: 31_536_000,
        age_seconds: 86_400,
        created_timestamp_seconds: 1_600_000_000,
        retrieved_at_timestamp_seconds: 1_700_000_000,
        voting_power: neuron_id.saturating_mul(200_000_000),
        deciding_voting_power: Some(neuron_id.saturating_mul(150_000_000)),
        potential_voting_power: Some(neuron_id.saturating_mul(200_000_000)),
        voting_power_refreshed_timestamp_seconds: Some(1_699_999_000),
        joined_community_fund_timestamp_seconds: None,
        eight_year_gang_bonus_base_e8s: None,
        known_neuron_data: Some(NnsKnownNeuronData {
            name: format!("Neuron {neuron_id}"),
            description: Some("fixture neuron".to_string()),
            links: vec!["https://example.com".to_string()],
        }),
        recent_ballots: Vec::new(),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn cache_rows_reject_unknown_fields_without_rewriting_evidence() {
    let root = temp_dir("ic-query-neuron-row-admission");
    let refresh = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    refresh_nns_neuron_cache_with_source(&refresh, &FixtureSource)
        .await
        .unwrap();
    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for mutate in [
        (|cache: &mut serde_json::Value| {
            cache["data"]["neurons"][0]["unexpected_evidence"] = serde_json::json!(true);
        }) as fn(&mut serde_json::Value),
        |cache| {
            cache["data"]["neurons"][0]["recent_ballots"] =
                serde_json::json!([{ "proposal_id": 7, "vote": 99, "unexpected_evidence": true }]);
        },
    ] {
        let mut cache = original.clone();
        mutate(&mut cache);
        let bytes = serde_json::to_vec(&cache).unwrap();
        fs::write(&path, &bytes).unwrap();
        let error = build_nns_neuron_distribution_report_from_cache(
            &NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            NnsNeuronHostError::Cache(crate::HostCacheError::ParseCache { .. })
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    refresh_nns_neuron_cache_with_source(&refresh, &FixtureSource)
        .await
        .unwrap();
    let restored = build_nns_neuron_distribution_report_from_cache(
        &NnsGovernanceCacheRequest::new(&root, MAINNET_NETWORK),
    )
    .unwrap();
    assert_eq!(restored.collected_neuron_count, 3);
    fs::remove_dir_all(root).unwrap();
}
