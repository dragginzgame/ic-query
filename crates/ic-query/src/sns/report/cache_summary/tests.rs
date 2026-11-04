use super::load_sns_cache_summaries;
use crate::cache::CacheValidationStatus;
use crate::{
    cache_file::{ManagedReadBudget, write_managed_text_atomically},
    sns::report::{
        cache_storage::sns_cache_scan_limit_exceeded, proposals_cache::SnsProposalsCacheCollection,
    },
    tests::temp_dir,
};

#[cfg(unix)]
use std::{ffi::OsString, os::unix::ffi::OsStringExt};

#[cfg(unix)]
#[test]
fn listing_respects_native_non_utf8_cache_root_support() {
    let parent = temp_dir("ic-query-sns-non-utf8-root");
    let root = parent.join(OsString::from_vec(b"cache-\xff".to_vec()));
    let path = root.join("sns/ic/aaaaa-aa/proposals/full.json");
    let written = write_managed_text_atomically(&root, &path, "{}");
    // macOS filesystems may reject the name with EILSEQ before a root exists.
    #[cfg(target_os = "macos")]
    if let Err(crate::cache_file::CacheFileError::CreateDirectory { path, source }) = &written
        && source.raw_os_error() == Some(92)
    {
        assert_eq!(path, &root);
        assert!(!root.exists());
        std::fs::remove_dir_all(parent).unwrap();
        return;
    }
    written.unwrap();
    let attempt = serde_json::json!({
        "schema_version": 1,
        "network": "ic",
        "source_endpoint": "https://icp-api.io",
        "started_at": "2026-10-24T00:00:00Z",
        "updated_at": "2026-10-24T00:00:00Z",
        "metadata": {
            "id": 1,
            "root_canister_id": "aaaaa-aa",
            "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai"
        },
        "status": "running",
        "page_size": 10,
        "pages_fetched": 0,
        "rows_fetched": 0,
        "last_cursor": null,
        "last_error": null
    });
    write_managed_text_atomically(
        &root,
        &path.with_file_name("full.refresh-attempt.json"),
        &attempt.to_string(),
    )
    .unwrap();
    let summaries = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
        &root,
        [path],
        "ic",
        &mut ManagedReadBudget::new(4096),
    )
    .unwrap();
    assert!(summaries[0].latest_attempt.is_some());
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn listing_budget_errors_are_not_invalid_rows_or_absent_attempts() {
    let root = temp_dir("ic-query-sns-list-budget");
    let path = root.join("sns/ic/aaaaa-aa/proposals/full.json");
    write_managed_text_atomically(&root, &path, "{}").unwrap();
    write_managed_text_atomically(
        &root,
        &path.with_file_name("full.refresh-attempt.json"),
        "{invalid",
    )
    .unwrap();
    let result = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
        &root,
        [path.clone()],
        "ic",
        &mut ManagedReadBudget::new(3),
    );
    assert!(sns_cache_scan_limit_exceeded(&result.unwrap_err()));
    let error = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
        &root,
        [path.clone()],
        "ic",
        &mut ManagedReadBudget::new(32),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        crate::sns::SnsHostError::Cache(crate::HostCacheError::ParseCache { .. })
    ));
    std::fs::remove_file(path.with_file_name("full.refresh-attempt.json")).unwrap();
    let summaries = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
        &root,
        [path],
        "ic",
        &mut ManagedReadBudget::new(32),
    )
    .unwrap();
    assert_eq!(summaries[0].cache_status(), CacheValidationStatus::Invalid);
    assert!(summaries[0].latest_attempt.is_none());
    std::fs::remove_dir_all(root).unwrap();
}
