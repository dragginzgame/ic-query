use super::{
    PagedCollectionState, SnapshotCompleteness, SnapshotEnvelope, SnapshotJsonPaths, SnapshotKey,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::Path;

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
        metadata: Metadata { id: 7 },
        completeness: SnapshotCompleteness::api_exhausted(100, 2, 101, false),
        data: Data {
            rows: vec!["row".to_string()],
        },
    };

    let value = serde_json::to_value(&envelope).expect("snapshot envelope serializes");
    assert_eq!(value["id"], 7);
    assert_eq!(value["rows"][0], "row");
    assert!(value.get("metadata").is_none());
    assert!(value.get("data").is_none());
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
