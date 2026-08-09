use super::source::validate_neuron_rows;
use super::{
    DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, NnsKnownNeuronData, NnsNeuronBallotRow, NnsNeuronHostError,
    NnsNeuronInfoRequest, NnsNeuronListRequest, NnsNeuronPage, NnsNeuronRow, NnsNeuronSource,
    NnsNeuronState, NnsNeuronType, NnsNeuronVisibility, NnsNeuronVote,
    build_nns_neuron_cache_status_report, build_nns_neuron_info_report_from_cache,
    build_nns_neuron_info_report_with_source, build_nns_neuron_list_report_from_cache,
    build_nns_neuron_list_report_with_source, nns_neuron_cache_path,
    nns_neuron_refresh_attempt_path, refresh_nns_neuron_cache_with_source,
};
use crate::{
    cache::{CacheRefreshAttemptStatus, CacheValidationStatus},
    nns::{
        LiveNnsSource, NnsGovernanceCacheRequest, NnsGovernanceRefreshRequest, NnsSourceRequest,
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

impl NnsNeuronSource for FixtureSource {
    fn fetch_neuron_page(
        &self,
        request: &NnsSourceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> Result<NnsNeuronPage, NnsNeuronHostError> {
        assert_eq!(request.network, MAINNET_NETWORK);
        assert_eq!(request.endpoint, DEFAULT_NNS_NEURON_SOURCE_ENDPOINT);
        assert_eq!(page_size, 2);
        let (neurons, next_start_neuron_id) = match exclusive_start_neuron_id {
            None => (vec![sample_neuron(1), sample_neuron(2)], Some(2)),
            Some(2) => (vec![sample_neuron(3)], None),
            other => panic!("unexpected neuron cursor: {other:?}"),
        };
        Ok(NnsNeuronPage {
            neurons,
            next_start_neuron_id,
        })
    }

    fn fetch_neuron(
        &self,
        request: &NnsSourceRequest,
        neuron_id: u64,
    ) -> Result<NnsNeuronRow, NnsNeuronHostError> {
        assert_eq!(request.network, MAINNET_NETWORK);
        Ok(sample_neuron(neuron_id))
    }
}

struct CountingSource;

impl NnsNeuronSource for CountingSource {
    fn fetch_neuron_page(
        &self,
        _request: &NnsSourceRequest,
        _exclusive_start_neuron_id: Option<u64>,
        _page_size: u32,
    ) -> Result<NnsNeuronPage, NnsNeuronHostError> {
        LIVE_SOURCE_CALLS.fetch_add(1, Ordering::SeqCst);
        unreachable!("unsupported network must be rejected before source invocation")
    }

    fn fetch_neuron(
        &self,
        _request: &NnsSourceRequest,
        _neuron_id: u64,
    ) -> Result<NnsNeuronRow, NnsNeuronHostError> {
        LIVE_SOURCE_CALLS.fetch_add(1, Ordering::SeqCst);
        unreachable!("unsupported network must be rejected before source invocation")
    }
}

#[test]
fn public_list_and_info_reports_preserve_governance_rows() {
    let list_request = NnsNeuronListRequest::new(
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    let list = build_nns_neuron_list_report_with_source(&list_request, &FixtureSource)
        .expect("live list report");

    assert!(!list.from_cache);
    assert_eq!(list.next_start_neuron_id, Some(2));
    assert_eq!(
        list.neurons
            .iter()
            .map(|row| row.neuron_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(list.neurons[0].visibility, Some(2));
    assert_eq!(list.neurons[0].visibility_text, NnsNeuronVisibility::Public);

    let info_request = NnsNeuronInfoRequest::new(
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        42,
    );
    let info = build_nns_neuron_info_report_with_source(&info_request, &FixtureSource)
        .expect("live info report");

    assert!(!info.from_cache);
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

#[test]
fn public_builders_reject_non_mainnet_before_invoking_a_source() {
    LIVE_SOURCE_CALLS.store(0, Ordering::SeqCst);
    let list_request = NnsNeuronListRequest::new("local", "http://127.0.0.1:1", 1_700_000_000, 2);
    let list_error = build_nns_neuron_list_report_with_source(&list_request, &CountingSource)
        .expect_err("non-mainnet list must fail");
    assert!(matches!(
        list_error,
        NnsNeuronHostError::UnsupportedNetwork { ref network } if network == "local"
    ));

    let info_request = NnsNeuronInfoRequest::new("local", "http://127.0.0.1:1", 1_700_000_000, 1);
    let info_error = build_nns_neuron_info_report_with_source(&info_request, &CountingSource)
        .expect_err("non-mainnet info must fail");
    assert!(matches!(
        info_error,
        NnsNeuronHostError::UnsupportedNetwork { ref network } if network == "local"
    ));
    assert_eq!(LIVE_SOURCE_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn live_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2023-11-14T22:13:20Z",
        "test",
    );

    let error = LiveNnsSource
        .fetch_neuron_page(&request, None, 2)
        .expect_err("live source must reject non-mainnet");

    assert!(matches!(
        error,
        NnsNeuronHostError::UnsupportedNetwork { ref network } if network == "local"
    ));
}

#[test]
fn list_rejects_a_cursor_that_does_not_match_the_page() {
    struct InvalidCursorSource;

    impl NnsNeuronSource for InvalidCursorSource {
        fn fetch_neuron_page(
            &self,
            _request: &NnsSourceRequest,
            _exclusive_start_neuron_id: Option<u64>,
            _page_size: u32,
        ) -> Result<NnsNeuronPage, NnsNeuronHostError> {
            Ok(NnsNeuronPage {
                neurons: vec![sample_neuron(1), sample_neuron(2)],
                next_start_neuron_id: Some(1),
            })
        }

        fn fetch_neuron(
            &self,
            _request: &NnsSourceRequest,
            neuron_id: u64,
        ) -> Result<NnsNeuronRow, NnsNeuronHostError> {
            Ok(sample_neuron(neuron_id))
        }
    }

    let request = NnsNeuronListRequest::new(
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    let error = build_nns_neuron_list_report_with_source(&request, &InvalidCursorSource)
        .expect_err("invalid cursor must fail");

    assert!(matches!(error, NnsNeuronHostError::InvalidPage { .. }));
}

#[test]
fn refresh_publishes_one_complete_snapshot_for_cached_list_and_info() {
    let root = temp_dir("ic-query-nns-neuron-cache");
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );

    let refresh = refresh_nns_neuron_cache_with_source(&refresh_request, &FixtureSource)
        .expect("refresh complete neuron cache");

    assert_eq!(refresh.neuron_count, 3);
    assert_eq!(refresh.page_count, 2);
    assert!(refresh.complete);
    assert!(!refresh.point_in_time_guaranteed);
    let path = nns_neuron_cache_path(&root, MAINNET_NETWORK);
    assert!(path.is_file());
    let cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("read neuron cache"))
            .expect("parse neuron cache");
    assert_eq!(cache["collection"], "neurons");
    assert_eq!(cache["completeness"]["status"], "api_exhausted");
    assert_eq!(cache["completeness"]["point_in_time_guaranteed"], false);

    let list_request = NnsNeuronListRequest::new(
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_001,
        2,
    )
    .with_exclusive_start_neuron_id(1);
    let list = build_nns_neuron_list_report_from_cache(&list_request, &root)
        .expect("load cached list")
        .expect("complete cache");
    assert!(list.from_cache);
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

    let info_request = NnsNeuronInfoRequest::new(
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_001,
        2,
    );
    let info = build_nns_neuron_info_report_from_cache(&info_request, &root)
        .expect("load cached detail")
        .expect("cached neuron");
    assert!(info.from_cache);
    assert_eq!(info.neuron.neuron_id, 2);

    let status = build_nns_neuron_cache_status_report(&NnsGovernanceCacheRequest::new(
        &root,
        MAINNET_NETWORK,
    ))
    .expect("cache status");
    assert!(status.found);
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

#[test]
fn cached_neuron_reports_return_typed_snapshot_identity_mismatches() {
    let root = temp_dir("ic-query-nns-neuron-cache-identity");
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    refresh_nns_neuron_cache_with_source(&refresh_request, &FixtureSource)
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

    let request = NnsNeuronListRequest::new(
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_001,
        2,
    );
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

#[test]
fn capped_refresh_keeps_failure_evidence_without_publishing_a_snapshot() {
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

#[test]
fn cache_status_distinguishes_invalid_attempt_evidence_from_the_snapshot() {
    let root = temp_dir("ic-query-nns-neuron-invalid-attempt");
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        MAINNET_NETWORK,
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );
    refresh_nns_neuron_cache_with_source(&refresh_request, &FixtureSource)
        .expect("refresh complete neuron cache");
    let attempt_path = nns_neuron_refresh_attempt_path(&root, MAINNET_NETWORK);
    let mut attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(&attempt_path).expect("read attempt"))
            .expect("parse attempt");
    attempt["governance_canister_id"] = serde_json::Value::String("aaaaa-aa".to_string());
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
        state_text: NnsNeuronState::NotDissolving,
        visibility: Some(2),
        visibility_text: NnsNeuronVisibility::Public,
        neuron_type: None,
        neuron_type_text: NnsNeuronType::Unknown,
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

#[test]
fn neuron_rows_reject_classifications_that_contradict_raw_codes() {
    let mut state_mismatch = sample_neuron(1);
    state_mismatch.state_text = NnsNeuronState::Dissolved;
    assert!(matches!(
        validate_neuron_rows(&[state_mismatch]),
        Err(NnsNeuronHostError::InvalidPage { reason })
            if reason.contains("state classification dissolved does not match raw code 1")
    ));

    let mut visibility_mismatch = sample_neuron(1);
    visibility_mismatch.visibility_text = NnsNeuronVisibility::Private;
    assert!(matches!(
        validate_neuron_rows(&[visibility_mismatch]),
        Err(NnsNeuronHostError::InvalidPage { reason })
            if reason.contains("visibility classification private does not match raw code Some(2)")
    ));

    let mut type_mismatch = sample_neuron(1);
    type_mismatch.neuron_type_text = NnsNeuronType::Seed;
    assert!(matches!(
        validate_neuron_rows(&[type_mismatch]),
        Err(NnsNeuronHostError::InvalidPage { reason })
            if reason.contains("type classification seed does not match raw code None")
    ));

    let mut vote_mismatch = sample_neuron(1);
    vote_mismatch.recent_ballots.push(NnsNeuronBallotRow {
        proposal_id: Some(7),
        vote: 1,
        vote_text: NnsNeuronVote::No,
    });
    assert!(matches!(
        validate_neuron_rows(&[vote_mismatch]),
        Err(NnsNeuronHostError::InvalidPage { reason })
            if reason.contains("ballot vote classification no does not match raw code 1")
    ));
}
