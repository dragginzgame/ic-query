use super::{fixtures::*, *};

#[test]
fn refresh_writes_catalog_atomically_and_removes_lock() {
    let root = temp_dir("ic-query-subnet-refresh");
    let mut catalog = fixture_catalog();
    catalog.registry_version = 987_654;
    catalog.fetched_at = "1970-01-01T00:00:00Z".to_string();
    catalog.source_endpoint = DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string();
    let source = FixtureRefreshSource::ok(catalog);
    let request = refresh_request(&root);

    let report = refresh_subnet_catalog_with_source(&request, &source).expect("refresh catalog");
    let cached = load_cached_subnet_catalog(&cache_request(&root)).expect("cached catalog");
    let lock_path = PathBuf::from(&report.refresh_lock_path);

    let _ = fs::remove_dir_all(root);
    assert!(report.wrote_catalog);
    assert!(!report.replaced_existing_catalog);
    assert_eq!(report.registry_version, 987_654);
    assert_eq!(cached.catalog.registry_version, 987_654);
    assert!(!lock_path.exists());
}

#[test]
fn refresh_dry_run_writes_output_without_replacing_cache() {
    let root = temp_dir("ic-query-subnet-refresh-dry-run");
    let mut catalog = fixture_catalog();
    catalog.fetched_at = "1970-01-01T00:00:00Z".to_string();
    catalog.source_endpoint = DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string();
    let output_path = root.join("catalog-export.json");
    let source = FixtureRefreshSource::ok(catalog);
    let mut request = refresh_request(&root);
    request.dry_run = true;
    request.output_path = Some(output_path.clone());

    let report = refresh_subnet_catalog_with_source(&request, &source).expect("dry-run");

    assert!(!report.wrote_catalog);
    assert!(!subnet_catalog_path(&request.cache.icp_root, MAINNET_NETWORK).exists());
    assert!(output_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn refresh_failure_preserves_existing_catalog_and_removes_lock() {
    let root = temp_dir("ic-query-subnet-refresh-failure");
    write_catalog(&root, fixture_catalog());
    let source = FixtureRefreshSource::err();
    let request = refresh_request(&root);

    let err = refresh_subnet_catalog_with_source(&request, &source).expect_err("refresh fails");
    let cached = load_cached_subnet_catalog(&cache_request(&root)).expect("cached catalog");
    let lock_path = subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK);

    assert!(matches!(
        err,
        SubnetCatalogHostError::InvalidStaleDuration { .. }
    ));
    assert_eq!(cached.catalog.registry_version, 123_456);
    assert!(!lock_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn refresh_existing_fresh_lock_fails_fast() {
    let root = temp_dir("ic-query-subnet-refresh-locked");
    let request = refresh_request(&root);
    let lock_path = subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK);
    write_refresh_lock_for_test(&lock_path, &request, request.now_unix_secs * 1_000);

    let err = refresh_subnet_catalog_with_source(&request, &FixtureRefreshSource::err())
        .expect_err("lock held");

    let _ = fs::remove_dir_all(root);
    assert!(matches!(
        err,
        SubnetCatalogHostError::RefreshAlreadyInProgress { .. }
    ));
}

#[test]
fn refresh_removes_stale_lock_and_retries_once() {
    let root = temp_dir("ic-query-subnet-refresh-stale-lock");
    let mut catalog = fixture_catalog();
    catalog.fetched_at = "1970-01-01T00:00:00Z".to_string();
    catalog.source_endpoint = DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string();
    let source = FixtureRefreshSource::ok(catalog);
    let request = refresh_request(&root);
    let lock_path = subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK);
    let stale_started_at = (request.now_unix_secs - request.lock_stale_after_seconds - 1) * 1_000;
    write_refresh_lock_for_test(&lock_path, &request, stale_started_at);

    let report = refresh_subnet_catalog_with_source(&request, &source).expect("stale lock removed");

    assert!(report.wrote_catalog);
    assert!(!lock_path.exists());
    let _ = fs::remove_dir_all(root);
}
