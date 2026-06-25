use super::{NnsLeafCacheRequest, NnsLeafRefreshRequest, write_nns_leaf_json_refresh_cache};
use crate::test_support::temp_dir;
use serde::Serialize;
use std::{fs, path::PathBuf};

#[test]
fn nns_leaf_refresh_writer_dry_run_writes_output_without_cache() {
    let root = temp_dir("ic-query-nns-leaf-refresh-dry-run");
    let output_path = root.join("exports").join("fixture.preview.json");
    let request = fixture_refresh_request(&root, true, Some(output_path.clone()));
    let report = FixtureReport {
        rows: vec!["dry-run"],
    };

    let result = write_nns_leaf_json_refresh_cache(&request, "fixture", "fixture.json", &report)
        .expect("dry-run refresh writes output");

    let cache_path = root
        .join(".icq")
        .join("fixture")
        .join("ic")
        .join("fixture.json");
    let lock_path = root
        .join(".icq")
        .join("fixture")
        .join("ic")
        .join("refresh.lock");
    assert_eq!(result.cache_path, cache_path.display().to_string());
    assert_eq!(result.refresh_lock_path, lock_path.display().to_string());
    assert_eq!(result.output_path, Some(output_path.display().to_string()));
    assert!(!result.replaced_existing_cache);
    assert!(!result.wrote_cache);
    assert!(!cache_path.exists());
    assert!(!lock_path.exists());
    assert!(
        fs::read_to_string(&output_path)
            .expect("read dry-run output")
            .contains("dry-run")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn nns_leaf_refresh_writer_replaces_component_cache_atomically() {
    let root = temp_dir("ic-query-nns-leaf-refresh-write");
    let request = fixture_refresh_request(&root, false, None);
    let first_report = FixtureReport {
        rows: vec!["first"],
    };
    let second_report = FixtureReport {
        rows: vec!["second"],
    };
    let cache_path = root
        .join(".icq")
        .join("fixture")
        .join("ic")
        .join("fixture.json");
    let lock_path = root
        .join(".icq")
        .join("fixture")
        .join("ic")
        .join("refresh.lock");

    let first =
        write_nns_leaf_json_refresh_cache(&request, "fixture", "fixture.json", &first_report)
            .expect("first refresh writes cache");
    let second =
        write_nns_leaf_json_refresh_cache(&request, "fixture", "fixture.json", &second_report)
            .expect("second refresh replaces cache");

    assert!(!first.replaced_existing_cache);
    assert!(first.wrote_cache);
    assert!(second.replaced_existing_cache);
    assert!(second.wrote_cache);
    assert_eq!(second.output_path, None);
    assert_eq!(second.cache_path, cache_path.display().to_string());
    assert_eq!(second.refresh_lock_path, lock_path.display().to_string());
    assert!(
        fs::read_to_string(&cache_path)
            .expect("read written cache")
            .contains("second")
    );
    assert!(!lock_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FixtureCacheRequest {
    icp_root: PathBuf,
    network: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FixtureRefreshRequest {
    cache: FixtureCacheRequest,
    source_endpoint: String,
    now_unix_secs: u64,
    lock_stale_after_seconds: u64,
    dry_run: bool,
    output_path: Option<PathBuf>,
}

impl_nns_leaf_cache_and_refresh_requests!(FixtureCacheRequest, FixtureRefreshRequest);

#[derive(Serialize)]
struct FixtureReport {
    rows: Vec<&'static str>,
}

fn fixture_refresh_request(
    root: &std::path::Path,
    dry_run: bool,
    output_path: Option<PathBuf>,
) -> FixtureRefreshRequest {
    FixtureRefreshRequest::from_leaf_parts(
        FixtureCacheRequest::from_root_network(root, "ic"),
        "https://icp-api.io".to_string(),
        1,
        60,
        dry_run,
        output_path,
    )
}
