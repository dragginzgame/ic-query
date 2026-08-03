#![cfg(feature = "host")]

use ic_query::cache::{
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheFileStatus, CacheRefreshLockStatus,
    CacheStatusRequest, build_cache_status_report, cache_status_report_text,
};
use std::path::PathBuf;

#[test]
fn public_cache_status_api_is_local_and_constructible() {
    let cache_root = PathBuf::from("target/ic-query-cache-public-api-empty-root");
    let request = CacheStatusRequest::new(&cache_root, 1_700_000_000);
    let report = build_cache_status_report(&request).expect("local cache inventory");

    assert_eq!(report.schema_version, CACHE_STATUS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.cache_root, cache_root.display().to_string());
    assert!(!report.cache_root_found);
    assert_eq!(report.cache_count, 0);
    assert_eq!(report.refresh_lock_count, 0);
    assert_eq!(CacheFileStatus::Unmanaged.as_str(), "unmanaged");
    assert_eq!(CacheRefreshLockStatus::Active.as_str(), "active");
    assert!(cache_status_report_text(&report).contains("cache_count: 0"));
}
