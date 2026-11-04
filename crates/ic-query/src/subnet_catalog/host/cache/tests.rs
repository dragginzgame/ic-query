use super::*;

#[test]
fn post_refresh_cache_load_failure_has_a_distinct_typed_stage_and_disposition() {
    let request = SubnetCatalogLoadRequest::cache_only(
        SubnetCatalogCacheRequest::new("/tmp/ic-query-post-refresh-test", "ic"),
        123,
    );
    let source = SubnetCatalogSourceFailure::new(
        Some(9001),
        Some(SubnetCatalogSubject::CachePath(PathBuf::from(
            "/tmp/ic-query-post-refresh-test/catalog.json",
        ))),
        SubnetCatalogHostError::MissingCatalog {
            path: PathBuf::from("/tmp/ic-query-post-refresh-test/catalog.json"),
        },
    );

    let failure = post_refresh_load_failure(&request, SubnetCatalogRefreshTrigger::Missing, source);

    assert_eq!(
        failure.stage,
        SubnetCatalogLoadStage::PostRefreshCacheLoadFailed
    );
    assert_eq!(
        failure.cache_disposition,
        SubnetCatalogFailureCacheDisposition::PostRefreshLoadFailed(
            SubnetCatalogRefreshTrigger::Missing
        )
    );
    assert_eq!(failure.evidence.registry_version, Some(9001));
}
