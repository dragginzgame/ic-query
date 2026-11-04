use super::*;
use crate::{
    cache::CacheCollectionCompleteness, cache_file::write_managed_text_atomically,
    sns::report::proposals_cache::SnsProposalsCacheCollection, tests::temp_dir,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn write_snapshot(root: &Path, entity: &str) -> PathBuf {
    let path = root.join("sns/ic").join(entity).join("proposals/full.json");
    let cache = serde_json::json!({
        "schema_version": 1, "network": "ic", "domain": "sns", "entity": entity,
        "collection": "proposals", "scope": "full", "id": 7, "name": "Fixture",
        "sns_wasm_canister_id": "qaa6y-5yaaa-aaaaa-aaafa-cai", "root_canister_id": entity,
        "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
        "source_endpoint": "https://icp-api.io", "fetched_at": "2026-10-03T00:00:00Z",
        "fetched_by": "fixture", "completeness": CacheCollectionCompleteness::api_exhausted(100, 1, 0, false),
        "proposals": [],
    });
    write_managed_text_atomically(root, &path, &cache.to_string()).unwrap();
    path
}

#[test]
fn root_status_reads_only_the_selected_snapshot() {
    let root = temp_dir("ic-query-sns-status-budget-invalid");
    write_snapshot(&root, "aaaaa-aa");
    let invalid = root.join("sns/ic/0-invalid/proposals/full.json");
    write_managed_text_atomically(&root, &invalid, "{invalid").unwrap();
    assert!(
        build_sns_cache_status_report::<SnsProposalsCacheCollection>(&SnsCacheStatusRequest::new(
            &root, "ic", "aaaaa-aa"
        ),)
        .unwrap()
        .cache
        .is_some()
    );
    fs::remove_dir_all(root).unwrap();
}
