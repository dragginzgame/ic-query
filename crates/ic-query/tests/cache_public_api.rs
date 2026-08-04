#[cfg(feature = "host")]
use ic_query::cache::{
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheRefreshLockStatus, CacheStatusRequest,
    build_cache_status_report, cache_status_report_text,
};
use ic_query::cache::{
    CacheAgeStatus, CacheCollectionCompleteness, CacheHeaderStatus, CacheRecoveryPolicy,
    CacheRefreshAttemptStatus, CacheValidationStatus, validate_cache_collection_completeness,
};
#[cfg(feature = "host")]
use std::path::PathBuf;

#[test]
fn public_cache_models_preserve_evidence_without_host() {
    assert_eq!(CacheValidationStatus::Valid.as_str(), "ok");
    assert_eq!(CacheValidationStatus::Invalid.as_str(), "invalid");
    assert_eq!(CacheRefreshAttemptStatus::Running.as_str(), "running");
    assert_eq!(CacheRefreshAttemptStatus::Complete.as_str(), "complete");
    assert_eq!(CacheRefreshAttemptStatus::Failed.as_str(), "failed");
    assert_eq!(CacheHeaderStatus::Readable.as_str(), "readable");
    assert_eq!(CacheAgeStatus::Unmanaged.as_str(), "unmanaged");
    assert_eq!(CacheRecoveryPolicy::Automatic.as_str(), "automatic");

    let completeness = CacheCollectionCompleteness::api_exhausted(100, 2, 101, false);
    assert!(completeness.is_api_exhausted());
    assert_eq!(
        validate_cache_collection_completeness(&completeness, 101),
        Ok(())
    );
    assert_eq!(
        serde_json::to_value(completeness).expect("serialize completeness")["status"],
        "api_exhausted"
    );
}

#[cfg(feature = "host")]
#[test]
fn public_cache_status_api_is_local_and_constructible() {
    let cache_root = PathBuf::from("target/ic-query-cache-public-api-empty-root");
    let request = CacheStatusRequest::new(&cache_root, 1_700_000_000);
    let mut report = build_cache_status_report(&request).expect("local cache inventory");

    assert_eq!(report.schema_version, CACHE_STATUS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.cache_root, cache_root.display().to_string());
    assert!(!report.cache_root_found);
    assert!(!report.family_validation_performed);
    assert_eq!(report.cache_count, 0);
    assert_eq!(report.refresh_lock_count, 0);
    assert_eq!(CacheRefreshLockStatus::Active.as_str(), "active");
    report.total_size_bytes = 1_372;
    report.refresh_lock_size_bytes = 1_024;
    let text = cache_status_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize cache status report");
    assert!(text.contains("cache_count: 0"));
    assert!(text.contains("family_validation_performed: no"));
    assert!(text.contains("total_size_bytes: 1.34 KiB"));
    assert!(text.contains("refresh_lock_size_bytes: 1 KiB"));
    assert_eq!(json["total_size_bytes"], 1_372);
    assert_eq!(json["refresh_lock_size_bytes"], 1_024);
    assert_eq!(json["family_validation_performed"], false);
}
