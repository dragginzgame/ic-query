use ic_query::cache::CacheCollectionCompleteness;
use ic_query::cache::CacheRefreshAttemptStatus;
use ic_query::cache::CacheValidationStatus;
use ic_query::cache::validate_cache_collection_completeness;

#[test]
fn public_cache_models_preserve_evidence_without_host() {
    assert_eq!(CacheValidationStatus::Valid.as_str(), "ok");
    assert_eq!(CacheValidationStatus::Invalid.as_str(), "invalid");
    assert_eq!(CacheRefreshAttemptStatus::Running.as_str(), "running");
    assert_eq!(CacheRefreshAttemptStatus::Complete.as_str(), "complete");
    assert_eq!(CacheRefreshAttemptStatus::Failed.as_str(), "failed");

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
