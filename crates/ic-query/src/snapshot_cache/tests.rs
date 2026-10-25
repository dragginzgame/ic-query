use super::{
    LockedSnapshotRefreshRequest, PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh,
    SnapshotEnvelope, SnapshotIdentityMismatch, SnapshotJsonPaths, SnapshotKey,
    SnapshotRefreshAttempt, SnapshotRefreshAttemptReadError,
    collect_full_collection_snapshot_paths, load_complete_snapshot_for_key,
    publish_snapshot_with_attempt, read_snapshot_refresh_attempt_strict,
    run_paged_snapshot_refresh_with_progress, run_snapshot_refresh_with_attempts,
    validate_snapshot_refresh_attempt, with_locked_snapshot_refresh,
};
use crate::{
    QueryProgressEvent, QueryProgressState,
    cache::{CacheCollectionCompleteness, CacheRefreshAttemptStatus},
    cache_file::{CacheFileError, HostCacheError, LoadJsonCacheRequest},
    test_support::temp_dir,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn shared_cache_rejects_oversize_before_json_and_classifies_invalid_bytes_as_content() {
    let root = temp_dir("ic-query-snapshot-byte-admission");
    let path = root.join("full.json");
    crate::cache_file::write_managed_file_atomically(&root, &path, |file| {
        std::io::Write::write_all(file, &[0xff])
    })
    .unwrap();
    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    assert!(matches!(
        load_fixture_snapshot(&path, &key).unwrap_err(),
        SnapshotLoadTestError::Cache(HostCacheError::ParseCache { path: actual, .. }) if actual == path
    ));
    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    assert!(matches!(
        load_fixture_snapshot(&path, &key).unwrap_err(),
        SnapshotLoadTestError::Cache(HostCacheError::CacheTooLarge { path: actual, actual: size, maximum, .. })
            if actual == path && size == 64 * 1024 * 1024 + 1 && maximum == 64 * 1024 * 1024
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshot_json_paths_encode_full_collection_scope() {
    let key = SnapshotKey::full("sns", "ic", "root-principal", "neurons");
    let paths = SnapshotJsonPaths::for_key(Path::new("/repo"), &key);

    assert_eq!(
        paths.snapshot_path,
        Path::new("/repo/sns/ic/root-principal/neurons/full.json")
    );
    assert_eq!(
        paths.refresh_lock_path,
        Path::new("/repo/sns/ic/root-principal/neurons/full.refresh.lock")
    );
    assert_eq!(
        paths.refresh_attempt_path,
        Path::new("/repo/sns/ic/root-principal/neurons/full.refresh-attempt.json")
    );
}

#[test]
fn collect_full_collection_snapshot_paths_lists_sorted_entity_snapshots() {
    let root = temp_dir("ic-query-snapshot-cache-path-scan");
    let network_dir = root.join("sns").join("ic");
    let b_path = network_dir.join("b-root").join("neurons").join("full.json");
    let a_path = network_dir.join("a-root").join("neurons").join("full.json");
    let ignored_path = network_dir.join("c-root").join("tokens").join("full.json");
    for path in [&b_path, &a_path, &ignored_path] {
        crate::cache_file::write_managed_text_atomically(&root, path, "{}")
            .expect("write snapshot fixture");
    }

    let paths = collect_full_collection_snapshot_paths(&root, &network_dir, "neurons")
        .expect("collect snapshot paths");

    assert_eq!(paths, vec![a_path, b_path]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn snapshot_envelope_serializes_nested_metadata_and_data() {
    #[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
    struct Metadata {
        id: usize,
    }

    #[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
    struct Data {
        rows: Vec<String>,
    }

    let envelope = SnapshotEnvelope {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-15T00:00:00Z".to_string(),
        fetched_by: "ic-query".to_string(),
        domain: "sns".to_string(),
        entity: "root".to_string(),
        collection: "neurons".to_string(),
        scope: "full".to_string(),
        metadata: Metadata { id: 7 },
        completeness: CacheCollectionCompleteness::api_exhausted(100, 2, 101, false),
        data: Data {
            rows: vec!["row".to_string()],
        },
    };

    let value = serde_json::to_value(&envelope).expect("snapshot envelope serializes");
    assert_eq!(value["domain"], "sns");
    assert_eq!(value["entity"], "root");
    assert_eq!(value["collection"], "neurons");
    assert_eq!(value["scope"], "full");
    assert_eq!(value["metadata"]["id"], 7);
    assert_eq!(value["data"]["rows"][0], "row");
}

#[test]
fn load_complete_snapshot_rejects_identity_mismatch() {
    let root = temp_dir("ic-query-snapshot-identity-mismatch");
    let path = root.join("full.json");
    write_snapshot_fixture(
        &path,
        serde_json::json!({
            "schema_version": 1,
            "network": "ic",
            "source_endpoint": "https://icp-api.io",
            "fetched_at": "2026-06-15T00:00:00Z",
            "fetched_by": "ic-query",
            "domain": "sns",
            "entity": "wrong-root",
            "collection": "neurons",
            "scope": "full",
            "metadata": {"id": 7},
            "completeness": {
                "status": "api_exhausted",
                "page_size": 100,
                "page_count": 2,
                "row_count": 101,
                "point_in_time_guaranteed": false
            },
            "data": {"rows": ["row"]}
        }),
    );

    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    let err = load_fixture_snapshot(&path, &key).expect_err("identity mismatch is rejected");

    assert!(matches!(
        err,
        SnapshotLoadTestError::Identity(SnapshotIdentityMismatch { field: "entity", expected, actual })
            if expected == "root" && actual == "wrong-root"
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn admitted_snapshot_handle_survives_path_replacement() {
    let root = temp_dir("ic-query-snapshot-admitted-handle");
    let path = root.join("full.json");
    write_snapshot_fixture(
        &path,
        serde_json::json!({
            "schema_version": 1, "network": "ic",
            "source_endpoint": "https://icp-api.io",
            "fetched_at": "2026-06-15T00:00:00Z", "fetched_by": "ic-query",
            "domain": "sns", "entity": "root", "collection": "neurons", "scope": "full",
            "metadata": {"id": 7},
            "completeness": {"status": "api_exhausted", "page_size": 100,
                "page_count": 1, "row_count": 1, "point_in_time_guaranteed": false},
            "data": {"rows": ["row"]}
        }),
    );
    let file = crate::cache_file::open_managed_file(&root, &path)
        .unwrap()
        .unwrap();
    write_snapshot_fixture(&path, serde_json::json!({}));
    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    let snapshot: FixtureSnapshot = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            component: "fixture",
            cache_root: &root,
            path: path.clone(),
            network: "ic",
            expected_schema_version: 1,
            maximum_bytes: 4096,
        },
        None,
        &key,
        SnapshotLoadTestError::Missing,
        |_| SnapshotLoadTestError::Incomplete,
        SnapshotLoadTestError::Identity,
        Some(file),
    )
    .unwrap();
    assert_eq!(snapshot.metadata.id, 7);
    assert!(matches!(
        load_fixture_snapshot(&path, &key),
        Err(SnapshotLoadTestError::Cache(
            HostCacheError::ParseCache { .. }
        ))
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn load_complete_snapshot_rejects_unknown_fields() {
    let root = temp_dir("ic-query-snapshot-unknown-field");
    let path = root.join("full.json");
    write_snapshot_fixture(
        &path,
        serde_json::json!({
            "schema_version": 1,
            "network": "ic",
            "source_endpoint": "https://icp-api.io",
            "fetched_at": "2026-06-15T00:00:00Z",
            "fetched_by": "ic-query",
            "domain": "sns",
            "entity": "root",
            "collection": "neurons",
            "scope": "full",
            "metadata": {"id": 7},
            "completeness": {
                "status": "api_exhausted",
                "page_size": 100,
                "page_count": 1,
                "row_count": 1,
                "point_in_time_guaranteed": false
            },
            "data": {"rows": ["row"]}
        }),
    );

    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    load_fixture_snapshot(&path, &key).expect("current nested snapshot is admitted");
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for object in [None, Some("metadata"), Some("data")] {
        let mut invalid = original.clone();
        let target = match object {
            Some(field) => &mut invalid[field],
            None => &mut invalid,
        };
        target["unexpected"] = serde_json::json!(true);
        write_snapshot_fixture(&path, invalid);
        let error = load_fixture_snapshot(&path, &key).expect_err("unknown field is rejected");
        assert!(
            matches!(error, SnapshotLoadTestError::Cache(HostCacheError::ParseCache { path: actual, .. }) if actual == path)
        );
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn load_complete_snapshot_rejects_unsupported_schema() {
    let root = temp_dir("ic-query-snapshot-unsupported-schema-before-rows");
    let path = root.join("full.json");
    write_snapshot_fixture(
        &path,
        serde_json::json!({
            "schema_version": 999,
            "network": "ic",
            "source_endpoint": "https://icp-api.io",
            "fetched_at": "2026-06-15T00:00:00Z",
            "fetched_by": "ic-query",
            "domain": "sns", "entity": "root", "collection": "neurons", "scope": "full",
            "metadata": {"id": 7},
            "completeness": {"status": "api_exhausted", "page_size": 100, "page_count": 1, "row_count": 1, "point_in_time_guaranteed": false},
            "data": {"rows": ["row"]}
        }),
    );

    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    let error = load_complete_snapshot_for_key::<FixtureSnapshotMetadata, FixtureSnapshotRows, _>(
        LoadJsonCacheRequest {
            component: "fixture",
            cache_root: &root,
            path,
            network: "ic",
            expected_schema_version: 1,
            maximum_bytes: 64 * 1024 * 1024,
        },
        None,
        &key,
        |path| HostCacheError::missing_cache("fixture", path).into(),
        |_| SnapshotLoadTestError::Incomplete,
        SnapshotLoadTestError::Identity,
        None,
    )
    .expect_err("unsupported schema is rejected");

    assert!(matches!(
        error,
        SnapshotLoadTestError::Cache(HostCacheError::UnsupportedCacheSchemaVersion {
            version: 999,
            expected: 1,
            ..
        })
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn snapshot_admission_preserves_missing_errors_and_validation_order() {
    let root = temp_dir("ic-query-snapshot-admission-order");
    let path = root.join("full.json");
    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    assert!(matches!(
        load_fixture_snapshot(&path, &key),
        Err(SnapshotLoadTestError::Missing(actual)) if actual == path
    ));

    let mut snapshot = serde_json::json!({
        "schema_version": 1,
        "network": "other",
        "source_endpoint": "https://icp-api.io",
        "fetched_at": "2026-06-15T00:00:00Z",
        "fetched_by": "ic-query",
        "domain": "sns", "entity": "wrong-root", "collection": "neurons", "scope": "full",
        "metadata": {"id": 7},
        "completeness": {
            "status": "unknown", "page_size": 1, "page_count": 1,
            "row_count": 1, "point_in_time_guaranteed": false
        },
        "data": {"rows": ["row"]}
    });
    write_snapshot_fixture(&path, snapshot.clone());
    assert!(matches!(
        load_fixture_snapshot(&path, &key),
        Err(SnapshotLoadTestError::Cache(
            HostCacheError::NetworkMismatch { .. }
        ))
    ));

    snapshot["network"] = "ic".into();
    write_snapshot_fixture(&path, snapshot.clone());
    assert!(matches!(
        load_fixture_snapshot(&path, &key),
        Err(SnapshotLoadTestError::Incomplete)
    ));

    snapshot["completeness"]["status"] = "api_exhausted".into();
    write_snapshot_fixture(&path, snapshot.clone());
    assert!(matches!(
        load_fixture_snapshot(&path, &key),
        Err(SnapshotLoadTestError::Identity(SnapshotIdentityMismatch {
            field: "entity",
            ..
        }))
    ));

    snapshot["entity"] = "root".into();
    write_snapshot_fixture(&path, snapshot);
    assert_eq!(
        load_fixture_snapshot(&path, &key).unwrap().data.rows,
        ["row"]
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshot_refresh_attempt_serializes_nested_metadata_and_validates_lifecycle() {
    #[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
    struct Metadata {
        root_canister_id: String,
    }

    let attempt = SnapshotRefreshAttempt {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        started_at: "2026-06-15T00:00:00Z".to_string(),
        updated_at: "2026-06-15T00:00:01Z".to_string(),
        metadata: Metadata {
            root_canister_id: "root".to_string(),
        },
        status: CacheRefreshAttemptStatus::Running,
        page_size: 100,
        pages_fetched: 1,
        rows_fetched: 25,
        last_cursor: Some("abcd".to_string()),
        last_error: None,
    };

    let value = serde_json::to_value(&attempt).expect("snapshot attempt serializes");
    assert_eq!(value["metadata"]["root_canister_id"], "root");
    assert_eq!(value["status"], "running");
    assert_eq!(value["rows_fetched"], 25);

    for (status, last_error) in [
        (CacheRefreshAttemptStatus::Running, None),
        (CacheRefreshAttemptStatus::Complete, None),
        (
            CacheRefreshAttemptStatus::Failed,
            Some("source failed".to_string()),
        ),
    ] {
        let mut candidate = attempt.clone();
        candidate.status = status;
        candidate.last_error = last_error;
        assert!(validate_snapshot_refresh_attempt(&candidate, "ic").is_ok());
    }

    let mut unknown = value;
    unknown["status"] = serde_json::json!("unknown");
    assert!(serde_json::from_value::<SnapshotRefreshAttempt<Metadata>>(unknown).is_err());

    for status in [
        CacheRefreshAttemptStatus::Running,
        CacheRefreshAttemptStatus::Complete,
    ] {
        let mut with_error = attempt.clone();
        with_error.status = status;
        with_error.last_error = Some("source failed".to_string());
        assert!(validate_snapshot_refresh_attempt(&with_error, "ic").is_err());
    }

    let mut blank_error = attempt.clone();
    blank_error.status = CacheRefreshAttemptStatus::Failed;
    blank_error.last_error = Some(" \t".to_string());
    assert!(validate_snapshot_refresh_attempt(&blank_error, "ic").is_err());

    let mut failed_without_error = attempt;
    failed_without_error.status = CacheRefreshAttemptStatus::Failed;
    assert_eq!(
        validate_snapshot_refresh_attempt(&failed_without_error, "ic"),
        Err("failed attempt must contain last_error".to_string())
    );
}

#[test]
fn snapshot_refresh_attempt_admits_nested_metadata_and_refuses_ambiguous_json() {
    let root = temp_dir("ic-query-attempt-json-admission");
    let path = root.join("full.refresh-attempt.json");
    let json = r#"{"schema_version":1,"network":"ic","source_endpoint":"https://icp-api.io",
        "started_at":"2026-06-15T00:00:00Z","updated_at":"2026-06-15T00:00:01Z",
        "status":"running","page_size":100,"pages_fetched":1,"rows_fetched":25,
        "last_cursor":null,"last_error":null,"metadata":{"id":7}}"#;
    let read = || {
        read_snapshot_refresh_attempt_strict::<SnapshotRefreshAttempt<FixtureSnapshotMetadata>>(
            &root, &path, None,
        )
    };
    crate::cache_file::write_managed_text_atomically(&root, &path, json).unwrap();
    let attempt = read().unwrap().unwrap();
    assert_eq!(attempt.metadata.id, 7);
    assert_eq!(attempt.status, CacheRefreshAttemptStatus::Running);
    assert_eq!(attempt.last_cursor, None);
    for duplicate in ["network", "metadata", "last_cursor"] {
        let value = match duplicate {
            "last_cursor" => "null",
            "metadata" => "{\"id\":7}",
            _ => "\"ic\"",
        };
        let invalid = format!("{{\"{duplicate}\":{value},{}", &json[1..]);
        crate::cache_file::write_managed_text_atomically(&root, &path, &invalid).unwrap();
        assert!(matches!(
            read(),
            Err(SnapshotRefreshAttemptReadError::Parse { .. })
        ));
    }
    let invalid = format!("{{\"unexpected\":0,{}", &json[1..]);
    crate::cache_file::write_managed_text_atomically(&root, &path, &invalid).unwrap();
    assert!(matches!(
        read(),
        Err(SnapshotRefreshAttemptReadError::Parse { .. })
    ));
    for invalid in [
        "[]",
        "{",
        &format!("{json} trailing"),
        &json.replace("\"id\":7", "\"id\":7,\"id\":8"),
        &json.replace("\"id\":7", "\"id\":7,\"unexpected\":true"),
    ] {
        crate::cache_file::write_managed_text_atomically(&root, &path, invalid).unwrap();
        assert!(matches!(
            read(),
            Err(SnapshotRefreshAttemptReadError::Parse { .. })
        ));
    }
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        json.replace("\"id\":7", "\"id\":7,\"unexpected\":true")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn snapshot_refresh_attempt_reader_rejects_oversized_sidecar() {
    let root = temp_dir("ic-query-oversized-refresh-attempt");
    let path = root.join("sns/ic/root/neurons/full.refresh-attempt.json");
    crate::cache_file::write_managed_text_atomically(&root, &path, &"x".repeat(1024 * 1024 + 1))
        .expect("write oversized attempt fixture");

    let error = read_snapshot_refresh_attempt_strict::<serde_json::Value>(&root, &path, None)
        .expect_err("oversized attempt is rejected");

    assert!(matches!(
        error,
        SnapshotRefreshAttemptReadError::Invalid { .. }
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn paged_collection_state_tracks_admitted_rows_and_original_page_length() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Row {
        id: &'static str,
    }

    let mut state = PagedCollectionState::<Row, Vec<u8>>::new();

    let first_page = state.ingest_page(vec![Row { id: "a" }], 2, Some(vec![1, 2]), |cursor| {
        format!("{cursor:?}")
    });

    assert_eq!(state.page_count(), 1);
    assert_eq!(state.row_count(), 1);
    assert_eq!(state.next_cursor(), Some(&vec![1, 2]));
    assert_eq!(first_page.last_cursor_text, Some("[1, 2]".to_string()));
    assert!(!first_page.exhausts_collection(2, state.has_next_cursor()));

    let final_page = state.ingest_page(vec![Row { id: "b" }], 1, None, |cursor| {
        format!("{cursor:?}")
    });

    assert_eq!(state.page_count(), 2);
    assert_eq!(state.row_count(), 2);
    assert!(final_page.exhausts_collection(2, state.has_next_cursor()));

    let complete = state.into_complete(|cursor| format!("{cursor:?}"));
    assert_eq!(complete.page_count, 2);
    assert_eq!(complete.rows, vec![Row { id: "a" }, Row { id: "b" }]);
    assert_eq!(complete.last_cursor, None);
}

#[tokio::test(flavor = "current_thread")]
async fn paged_snapshot_refresh_runner_fetches_until_collection_exhaustion() {
    let refresh = FixturePagedRefresh {
        pages: vec![(vec!["a"], 2, Some("next")), (vec!["b"], 1, None)],
        max_pages: None,
        attempts: RefCell::new(Vec::new()),
        attempt_error: None,
        state: PagedCollectionState::new(),
    };

    let mut events = Vec::new();
    let complete = run_paged_snapshot_refresh_with_progress(refresh, &mut |event| {
        events.push(event);
    })
    .await
    .expect("paged refresh completes");

    assert_eq!(complete.rows, vec!["a", "b"]);
    assert_eq!(complete.attempts, vec![(1, 1), (2, 2)]);
    assert_eq!(
        events,
        vec![
            QueryProgressEvent::PagedRefresh {
                text: "fixture pages=0 rows=0".to_string(),
                state: QueryProgressState::Running,
            },
            QueryProgressEvent::PagedRefresh {
                text: "fixture pages=1 rows=1".to_string(),
                state: QueryProgressState::Running,
            },
            QueryProgressEvent::PagedRefresh {
                text: "fixture pages=2 rows=2".to_string(),
                state: QueryProgressState::Running,
            },
            QueryProgressEvent::PagedRefresh {
                text: "fixture pages=2 rows=2 complete".to_string(),
                state: QueryProgressState::Complete,
            },
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn paged_snapshot_refresh_runner_stops_at_max_pages_before_next_fetch() {
    let refresh = FixturePagedRefresh {
        pages: vec![(vec!["a", "b"], 2, Some("next")), (vec!["c"], 1, None)],
        max_pages: Some(1),
        attempts: RefCell::new(Vec::new()),
        attempt_error: None,
        state: PagedCollectionState::new(),
    };

    let err = run_paged_snapshot_refresh_with_progress(refresh, &mut |_| {})
        .await
        .expect_err("max pages rejects incomplete scan");

    assert_eq!(
        err,
        "max pages reached before API exhaustion after 1 pages and 2 rows"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn paged_snapshot_refresh_rejects_nonterminal_page_without_admitted_rows() {
    let refresh = FixturePagedRefresh {
        pages: vec![
            (vec!["a", "b"], 2, Some("next")),
            (vec![], 2, Some("still-next")),
        ],
        max_pages: None,
        attempts: RefCell::new(Vec::new()),
        attempt_error: None,
        state: PagedCollectionState::new(),
    };

    let err = run_paged_snapshot_refresh_with_progress(refresh, &mut |_| {})
        .await
        .expect_err("stalled page is incomplete");

    assert_eq!(
        err,
        "page returned no new rows while advertising another cursor after 2 pages and 2 rows"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn paged_refresh_reports_fetch_and_checkpoint_failures_without_completing() {
    for (pages, attempt_error, expected_error, expected_progress) in [
        (
            vec![],
            None,
            "no fixture page",
            "fixture pages=0 rows=0 failed",
        ),
        (
            vec![(vec!["a"], 1, None)],
            Some("checkpoint failed"),
            "checkpoint failed",
            "fixture pages=1 rows=1 failed",
        ),
    ] {
        let refresh = FixturePagedRefresh {
            pages,
            max_pages: None,
            attempts: RefCell::new(Vec::new()),
            attempt_error,
            state: PagedCollectionState::new(),
        };
        let mut events = Vec::new();
        let error = run_paged_snapshot_refresh_with_progress(refresh, &mut |event| {
            events.push(event);
        })
        .await
        .unwrap_err();
        assert_eq!(error, expected_error);
        assert_eq!(
            events,
            [
                QueryProgressEvent::PagedRefresh {
                    text: "fixture pages=0 rows=0".to_string(),
                    state: QueryProgressState::Running,
                },
                QueryProgressEvent::PagedRefresh {
                    text: expected_progress.to_string(),
                    state: QueryProgressState::Failed,
                },
            ]
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn snapshot_refresh_lifecycle_preserves_original_error_after_failed_attempt_write() {
    let events = RefCell::new(Vec::new());

    let result: Result<(), &str> = run_snapshot_refresh_with_attempts(
        || {
            events.borrow_mut().push("running");
            Ok(())
        },
        async || {
            events.borrow_mut().push("refresh");
            Err("source failed")
        },
        |err| {
            events.borrow_mut().push(*err);
            events.borrow_mut().push("failed-attempt");
        },
    )
    .await;

    assert_eq!(result, Err("source failed"));
    assert_eq!(
        events.into_inner(),
        vec!["running", "refresh", "source failed", "failed-attempt"]
    );
}

#[test]
fn committed_snapshot_reports_attempt_finalization_failure_without_becoming_failed() {
    let events = RefCell::new(Vec::new());

    let result: Result<_, &str> = publish_snapshot_with_attempt(
        || {
            events.borrow_mut().push("snapshot");
            Ok(())
        },
        || {
            events.borrow_mut().push("attempt");
            Err("attempt write failed")
        },
    );

    let attempt_error = result.expect("the committed snapshot remains successful");
    assert_eq!(attempt_error.as_deref(), Some("attempt write failed"));
    assert_eq!(events.into_inner(), vec!["snapshot", "attempt"]);
}

#[test]
fn failed_snapshot_publication_does_not_finalize_the_attempt() {
    let events = RefCell::new(Vec::new());

    let result: Result<_, &str> = publish_snapshot_with_attempt(
        || {
            events.borrow_mut().push("snapshot");
            Err("snapshot write failed")
        },
        || {
            events.borrow_mut().push("attempt");
            Ok(())
        },
    );

    assert_eq!(result, Err("snapshot write failed"));
    assert_eq!(events.into_inner(), vec!["snapshot"]);
}

#[tokio::test(flavor = "current_thread")]
async fn locked_snapshot_refresh_creates_parent_tracks_replacement_and_releases_lock() {
    let root = temp_dir("ic-query-snapshot-cache-locked-refresh");
    let snapshot_path = root
        .join("sns")
        .join("ic")
        .join("root")
        .join("neurons")
        .join("full.json");
    let lock_path = snapshot_path.with_file_name("full.refresh.lock");
    let observed = RefCell::new(Vec::new());

    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &root,
            snapshot_path: &snapshot_path,
            refresh_lock_path: &lock_path,
        },
        identity_cache_error,
        async |state| {
            assert!(lock_path.is_file());
            let competing = with_locked_snapshot_refresh(
                LockedSnapshotRefreshRequest {
                    cache_root: &root,
                    snapshot_path: &snapshot_path,
                    refresh_lock_path: &lock_path,
                },
                identity_cache_error,
                async |_| panic!("competing refresh must not run"),
            )
            .await;
            assert!(matches!(
                competing,
                Err::<(), _>(CacheFileError::RefreshAlreadyInProgress { .. })
            ));
            observed.borrow_mut().push(state.replaced_existing_snapshot);
            crate::cache_file::write_managed_text_atomically(&root, &snapshot_path, "{}")
                .expect("write snapshot during refresh");
            Ok(())
        },
    )
    .await
    .expect("first locked refresh");

    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &root,
            snapshot_path: &snapshot_path,
            refresh_lock_path: &lock_path,
        },
        identity_cache_error,
        async |state| {
            observed.borrow_mut().push(state.replaced_existing_snapshot);
            Ok(())
        },
    )
    .await
    .expect("second locked refresh");

    assert_eq!(observed.into_inner(), vec![false, true]);
    assert!(snapshot_path.is_file());
    assert!(!crate::cache_file::inspect_refresh_lock(&root, &(lock_path)).unwrap());
    let _ = fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn locked_snapshot_refresh_releases_lock_after_target_inspection_failure() {
    let root = temp_dir("ic-query-snapshot-cache-invalid-target");
    let snapshot_path = root.join("full.json");
    let lock_path = root.join("full.refresh.lock");
    crate::cache_file::create_managed_parent_directory(&root, &snapshot_path.join("probe"))
        .expect("create nonregular snapshot target");
    let result = with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &root,
            snapshot_path: &snapshot_path,
            refresh_lock_path: &lock_path,
        },
        |error| {
            assert!(
                lock_path.is_file(),
                "target inspection occurs under the lock"
            );
            error
        },
        async |_| panic!("invalid target must not start refresh"),
    )
    .await;
    assert!(matches!(
        result,
        Err::<(), _>(CacheFileError::Confinement { .. })
    ));
    assert!(!crate::cache_file::inspect_refresh_lock(&root, &(lock_path)).unwrap());
    fs::remove_dir_all(root).unwrap();
}

const fn identity_cache_error(err: CacheFileError) -> CacheFileError {
    err
}

#[derive(Debug)]
enum SnapshotLoadTestError {
    Cache(HostCacheError),
    Missing(PathBuf),
    Incomplete,
    Identity(SnapshotIdentityMismatch),
}

impl From<HostCacheError> for SnapshotLoadTestError {
    fn from(error: HostCacheError) -> Self {
        Self::Cache(error)
    }
}

#[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureSnapshotMetadata {
    id: usize,
}

#[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureSnapshotRows {
    rows: Vec<String>,
}

type FixtureSnapshot = SnapshotEnvelope<FixtureSnapshotMetadata, FixtureSnapshotRows>;

fn load_fixture_snapshot(
    path: &Path,
    key: &SnapshotKey,
) -> Result<FixtureSnapshot, SnapshotLoadTestError> {
    load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            component: "fixture",
            cache_root: path.parent().expect("fixture cache root"),
            path: path.to_path_buf(),
            network: "ic",
            expected_schema_version: 1,
            maximum_bytes: 64 * 1024 * 1024,
        },
        None,
        key,
        SnapshotLoadTestError::Missing,
        |_| SnapshotLoadTestError::Incomplete,
        SnapshotLoadTestError::Identity,
        None,
    )
}

fn write_snapshot_fixture(path: &Path, value: serde_json::Value) {
    let cache_root = path.parent().expect("snapshot fixture parent");
    crate::cache_file::write_managed_text_atomically(
        cache_root,
        path,
        &serde_json::to_string_pretty(&value).expect("serialize snapshot fixture"),
    )
    .expect("write snapshot fixture");
}

struct FixturePagedRefresh {
    pages: Vec<(Vec<&'static str>, usize, Option<&'static str>)>,
    max_pages: Option<u32>,
    attempts: RefCell<Vec<(u32, usize)>>,
    attempt_error: Option<&'static str>,
    state: PagedCollectionState<&'static str, &'static str>,
}

#[derive(Debug, Eq, PartialEq)]
struct FixturePagedComplete {
    rows: Vec<&'static str>,
    attempts: Vec<(u32, usize)>,
}

impl PagedSnapshotRefresh for FixturePagedRefresh {
    type Complete = FixturePagedComplete;
    type Error = String;

    fn progress_text(&self) -> String {
        format!(
            "fixture pages={} rows={}",
            self.state.page_count(),
            self.state.row_count()
        )
    }

    fn max_pages_reached(&self) -> bool {
        self.max_pages
            .is_some_and(|max_pages| self.state.page_count() >= max_pages)
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        format!(
            "{reason} after {} pages and {} rows",
            self.state.page_count(),
            self.state.row_count()
        )
    }

    fn fetch_next_page(
        &mut self,
    ) -> crate::report::SourceFuture<'_, Result<PagedCollectionPage, Self::Error>> {
        Box::pin(async move {
            if self.pages.is_empty() {
                return Err("no fixture page".to_string());
            }
            let (rows, page_len, cursor) = self.pages.remove(0);
            Ok(self
                .state
                .ingest_page(rows, page_len, cursor, |cursor| (*cursor).to_string()))
        })
    }

    fn write_running_attempt(&self, _page: &PagedCollectionPage) -> Result<(), Self::Error> {
        self.attempts
            .borrow_mut()
            .push((self.state.page_count(), self.state.row_count()));
        self.attempt_error
            .map_or(Ok(()), |error| Err(error.to_string()))
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(2, self.state.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        FixturePagedComplete {
            rows: self
                .state
                .into_complete(|cursor| (*cursor).to_string())
                .rows,
            attempts: self.attempts.into_inner(),
        }
    }
}
