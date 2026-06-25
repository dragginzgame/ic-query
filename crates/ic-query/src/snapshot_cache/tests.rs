use super::{
    LockedSnapshotRefreshRequest, PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh,
    SnapshotCompleteness, SnapshotEnvelope, SnapshotIdentityMismatch, SnapshotJsonPaths,
    SnapshotKey, SnapshotRefreshAttempt, collect_full_collection_snapshot_paths,
    load_complete_snapshot_for_key, run_paged_snapshot_refresh, run_snapshot_refresh_with_attempts,
    with_locked_snapshot_refresh,
};
use crate::{
    cache_file::{CacheFileError, LoadJsonCacheErrorMapper, LoadJsonCacheRequest},
    test_support::temp_dir,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::{
    cell::RefCell,
    fs, io,
    path::{Path, PathBuf},
};

#[test]
fn snapshot_json_paths_encode_full_collection_scope() {
    let key = SnapshotKey::full("sns", "ic", "root-principal", "neurons");
    let paths = SnapshotJsonPaths::for_key(Path::new("/repo"), &key);

    assert_eq!(
        paths.snapshot_path,
        Path::new("/repo/.icq/sns/ic/root-principal/neurons/full.json")
    );
    assert_eq!(
        paths.refresh_lock_path,
        Path::new("/repo/.icq/sns/ic/root-principal/neurons/full.refresh.lock")
    );
    assert_eq!(
        paths.refresh_attempt_path,
        Path::new("/repo/.icq/sns/ic/root-principal/neurons/full.refresh-attempt.json")
    );
}

#[test]
fn collect_full_collection_snapshot_paths_lists_sorted_entity_snapshots() {
    let root = temp_dir("ic-query-snapshot-cache-path-scan");
    let network_dir = root.join(".icq").join("sns").join("ic");
    let b_path = network_dir.join("b-root").join("neurons").join("full.json");
    let a_path = network_dir.join("a-root").join("neurons").join("full.json");
    let ignored_path = network_dir.join("c-root").join("tokens").join("full.json");
    fs::create_dir_all(b_path.parent().expect("b snapshot parent")).expect("create b snapshot dir");
    fs::create_dir_all(a_path.parent().expect("a snapshot parent")).expect("create a snapshot dir");
    fs::create_dir_all(ignored_path.parent().expect("ignored snapshot parent"))
        .expect("create ignored snapshot dir");
    fs::write(&b_path, "{}").expect("write b snapshot");
    fs::write(&a_path, "{}").expect("write a snapshot");
    fs::write(&ignored_path, "{}").expect("write ignored snapshot");

    let paths = collect_full_collection_snapshot_paths(&network_dir, "neurons")
        .expect("collect snapshot paths");

    assert_eq!(paths, vec![a_path, b_path]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn snapshot_envelope_serializes_flat_metadata_and_data() {
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
        domain: Some("sns".to_string()),
        entity: Some("root".to_string()),
        collection: Some("neurons".to_string()),
        scope: Some("full".to_string()),
        metadata: Metadata { id: 7 },
        completeness: SnapshotCompleteness::api_exhausted(100, 2, 101, false),
        data: Data {
            rows: vec!["row".to_string()],
        },
    };

    let value = serde_json::to_value(&envelope).expect("snapshot envelope serializes");
    assert_eq!(value["domain"], "sns");
    assert_eq!(value["entity"], "root");
    assert_eq!(value["collection"], "neurons");
    assert_eq!(value["scope"], "full");
    assert_eq!(value["id"], 7);
    assert_eq!(value["rows"][0], "row");
    assert!(value.get("metadata").is_none());
    assert!(value.get("data").is_none());
}

#[test]
fn snapshot_envelope_deserializes_legacy_cache_without_identity() {
    #[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
    struct Metadata {
        id: usize,
    }

    #[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
    struct Data {
        rows: Vec<String>,
    }

    let envelope: SnapshotEnvelope<Metadata, Data> = serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "network": "ic",
        "source_endpoint": "https://icp-api.io",
        "fetched_at": "2026-06-15T00:00:00Z",
        "fetched_by": "ic-query",
        "id": 7,
        "completeness": {
            "status": "api_exhausted",
            "page_size": 100,
            "page_count": 2,
            "row_count": 101,
            "point_in_time_guaranteed": false
        },
        "rows": ["row"]
    }))
    .expect("legacy snapshot envelope deserializes");

    assert_eq!(envelope.domain, None);
    assert_eq!(envelope.entity, None);
    assert_eq!(envelope.collection, None);
    assert_eq!(envelope.scope, None);
    assert_eq!(envelope.metadata.id, 7);
    assert_eq!(envelope.data.rows, vec!["row".to_string()]);
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
            "id": 7,
            "completeness": {
                "status": "api_exhausted",
                "page_size": 100,
                "page_count": 2,
                "row_count": 101,
                "point_in_time_guaranteed": false
            },
            "rows": ["row"]
        }),
    );

    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    let err = load_fixture_snapshot(&path, &key).expect_err("identity mismatch is rejected");

    assert_eq!(
        err,
        SnapshotLoadTestError::Identity(SnapshotIdentityMismatch {
            field: "entity",
            expected: "root".to_string(),
            actual: "wrong-root".to_string(),
        })
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn load_complete_snapshot_allows_legacy_snapshot_without_identity() {
    let root = temp_dir("ic-query-snapshot-legacy-identity");
    let path = root.join("full.json");
    write_snapshot_fixture(
        &path,
        serde_json::json!({
            "schema_version": 1,
            "network": "ic",
            "source_endpoint": "https://icp-api.io",
            "fetched_at": "2026-06-15T00:00:00Z",
            "fetched_by": "ic-query",
            "id": 7,
            "completeness": {
                "status": "api_exhausted",
                "page_size": 100,
                "page_count": 2,
                "row_count": 101,
                "point_in_time_guaranteed": false
            },
            "rows": ["row"]
        }),
    );

    let key = SnapshotKey::full("sns", "ic", "root", "neurons");
    let loaded = load_fixture_snapshot(&path, &key).expect("legacy snapshot loads");

    assert_eq!(loaded.metadata.id, 7);
    assert_eq!(loaded.data.rows, vec!["row".to_string()]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn snapshot_refresh_attempt_serializes_flat_metadata() {
    #[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
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
        status: "running".to_string(),
        page_size: 100,
        pages_fetched: 1,
        rows_fetched: 25,
        last_cursor: Some("abcd".to_string()),
        last_error: None,
    };

    let value = serde_json::to_value(&attempt).expect("snapshot attempt serializes");
    assert_eq!(value["root_canister_id"], "root");
    assert_eq!(value["status"], "running");
    assert_eq!(value["rows_fetched"], 25);
    assert!(value.get("metadata").is_none());
}

#[test]
fn paged_collection_state_tracks_progress_and_deduplicates_rows() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Row {
        id: &'static str,
    }

    let mut state = PagedCollectionState::<Row, Vec<u8>>::new();

    let first_page = state.ingest_page(
        vec![Row { id: "a" }, Row { id: "a" }],
        Some(vec![1, 2]),
        |cursor| format!("{cursor:?}"),
        |row| row.id.to_string(),
    );

    assert_eq!(state.page_count(), 1);
    assert_eq!(state.row_count(), 1);
    assert_eq!(state.next_cursor(), Some(&vec![1, 2]));
    assert_eq!(first_page.last_cursor_text, Some("[1, 2]".to_string()));
    assert!(!first_page.exhausts_collection(2, state.has_next_cursor()));

    let final_page = state.ingest_page(
        vec![Row { id: "a" }],
        None,
        |cursor| format!("{cursor:?}"),
        |row| row.id.to_string(),
    );

    assert_eq!(state.page_count(), 2);
    assert_eq!(state.row_count(), 1);
    assert!(final_page.exhausts_collection(2, state.has_next_cursor()));

    let complete = state.into_complete(|cursor| format!("{cursor:?}"));
    assert_eq!(complete.page_count, 2);
    assert_eq!(complete.rows, vec![Row { id: "a" }]);
    assert_eq!(complete.last_cursor, None);
}

#[test]
fn paged_snapshot_refresh_runner_fetches_until_collection_exhaustion() {
    let refresh = FixturePagedRefresh {
        pages: vec![(vec!["a", "a"], Some("next")), (vec!["b"], None)],
        max_pages: None,
        attempts: RefCell::new(Vec::new()),
        state: PagedCollectionState::new(),
    };

    let complete = run_paged_snapshot_refresh(refresh).expect("paged refresh completes");

    assert_eq!(complete.rows, vec!["a", "b"]);
    assert_eq!(complete.attempts, vec![(1, 1), (2, 2)]);
}

#[test]
fn paged_snapshot_refresh_runner_stops_at_max_pages_before_next_fetch() {
    let refresh = FixturePagedRefresh {
        pages: vec![(vec!["a", "b"], Some("next")), (vec!["c"], None)],
        max_pages: Some(1),
        attempts: RefCell::new(Vec::new()),
        state: PagedCollectionState::new(),
    };

    let err = run_paged_snapshot_refresh(refresh).expect_err("max pages rejects incomplete scan");

    assert_eq!(err, "max pages reached after 1 pages and 2 rows");
}

#[test]
fn snapshot_refresh_lifecycle_preserves_original_error_after_failed_attempt_write() {
    let events = RefCell::new(Vec::new());

    let result: Result<(), &str> = run_snapshot_refresh_with_attempts(
        || {
            events.borrow_mut().push("running");
            Ok(())
        },
        || {
            events.borrow_mut().push("refresh");
            Err("source failed")
        },
        |err| {
            events.borrow_mut().push(*err);
            events.borrow_mut().push("failed-attempt");
        },
    );

    assert_eq!(result, Err("source failed"));
    assert_eq!(
        events.into_inner(),
        vec!["running", "refresh", "source failed", "failed-attempt"]
    );
}

#[test]
fn locked_snapshot_refresh_creates_parent_tracks_replacement_and_releases_lock() {
    let root = temp_dir("ic-query-snapshot-cache-locked-refresh");
    let snapshot_path = root
        .join(".icq")
        .join("sns")
        .join("ic")
        .join("root")
        .join("neurons")
        .join("full.json");
    let lock_path = snapshot_path.with_file_name("full.refresh.lock");
    let observed = RefCell::new(Vec::new());

    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &snapshot_path,
            refresh_lock_path: &lock_path,
            network: "ic",
            now_unix_secs: 1,
            lock_stale_after_seconds: 60,
        },
        identity_cache_error,
        |state| {
            observed.borrow_mut().push(state.replaced_existing_snapshot);
            fs::write(&snapshot_path, "{}").expect("write snapshot during refresh");
            Ok(())
        },
    )
    .expect("first locked refresh");

    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &snapshot_path,
            refresh_lock_path: &lock_path,
            network: "ic",
            now_unix_secs: 2,
            lock_stale_after_seconds: 60,
        },
        identity_cache_error,
        |state| {
            observed.borrow_mut().push(state.replaced_existing_snapshot);
            Ok(())
        },
    )
    .expect("second locked refresh");

    assert_eq!(observed.into_inner(), vec![false, true]);
    assert!(snapshot_path.is_file());
    assert!(!lock_path.exists());
    let _ = fs::remove_dir_all(root);
}

fn identity_cache_error(err: CacheFileError) -> CacheFileError {
    err
}

#[derive(Debug, Eq, PartialEq)]
enum SnapshotLoadTestError {
    Missing(PathBuf),
    Read(PathBuf),
    Parse(PathBuf),
    UnsupportedSchema { version: u32, expected: u32 },
    NetworkMismatch { requested: String, actual: String },
    Incomplete,
    Identity(SnapshotIdentityMismatch),
}

struct SnapshotLoadTestErrors;

impl LoadJsonCacheErrorMapper for SnapshotLoadTestErrors {
    type Error = SnapshotLoadTestError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        SnapshotLoadTestError::Missing(path)
    }

    fn read_cache(&self, path: PathBuf, _source: io::Error) -> Self::Error {
        SnapshotLoadTestError::Read(path)
    }

    fn parse_cache(&self, path: PathBuf, _source: serde_json::Error) -> Self::Error {
        SnapshotLoadTestError::Parse(path)
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        SnapshotLoadTestError::UnsupportedSchema { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        SnapshotLoadTestError::NetworkMismatch { requested, actual }
    }
}

#[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct FixtureSnapshotMetadata {
    id: usize,
}

#[derive(Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
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
            path: path.to_path_buf(),
            network: "ic",
            expected_schema_version: 1,
        },
        key,
        SnapshotLoadTestErrors,
        |_| SnapshotLoadTestError::Incomplete,
        SnapshotLoadTestError::Identity,
    )
}

fn write_snapshot_fixture(path: &Path, value: serde_json::Value) {
    fs::create_dir_all(path.parent().expect("snapshot fixture parent"))
        .expect("create snapshot fixture parent");
    fs::write(
        path,
        serde_json::to_vec_pretty(&value).expect("serialize snapshot fixture"),
    )
    .expect("write snapshot fixture");
}

struct FixturePagedRefresh {
    pages: Vec<(Vec<&'static str>, Option<&'static str>)>,
    max_pages: Option<u32>,
    attempts: RefCell<Vec<(u32, usize)>>,
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

    fn incomplete_refresh_error(&self) -> Self::Error {
        format!(
            "max pages reached after {} pages and {} rows",
            self.state.page_count(),
            self.state.row_count()
        )
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        if self.pages.is_empty() {
            return Err("no fixture page".to_string());
        }
        let (rows, cursor) = self.pages.remove(0);
        Ok(self.state.ingest_page(
            rows,
            cursor,
            |cursor| (*cursor).to_string(),
            |row| (*row).to_string(),
        ))
    }

    fn write_running_attempt(&self, _page: &PagedCollectionPage) -> Result<(), Self::Error> {
        self.attempts
            .borrow_mut()
            .push((self.state.page_count(), self.state.row_count()));
        Ok(())
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
