use super::{fixtures::*, *};

#[test]
fn catalog_progress_tracks_live_policy_and_not_cache_hits_or_cache_only() {
    let root = temp_dir("ic-query-catalog-progress");
    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    let source = FixtureRefreshSource::ok(fixture_catalog());
    let mut request = list_request(&root);
    let mut events = Vec::new();
    for disposition in [
        CacheDisposition::RefreshedMissing,
        CacheDisposition::RefreshedInvalid,
        CacheDisposition::RefreshedStale,
        CacheDisposition::ForcedRefresh,
    ] {
        match disposition {
            CacheDisposition::RefreshedInvalid => {
                crate::cache_file::write_managed_text_atomically(&root, &path, "not-json").unwrap();
            }
            CacheDisposition::RefreshedStale => {
                request.now_unix_secs += 100;
                request.read_policy = CatalogReadPolicy::RefreshMissingInvalidOrOlderThan {
                    source: CatalogSourceSelection::uncertified_query(
                        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
                    ),
                    max_age_seconds: 1,
                };
            }
            CacheDisposition::ForcedRefresh => {
                request.read_policy = CatalogReadPolicy::ForceRefresh {
                    source: CatalogSourceSelection::uncertified_query(
                        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
                    ),
                };
            }
            _ => {}
        }
        events.clear();
        let report = build_subnet_catalog_list_report_with_source_and_progress(
            &request,
            &source,
            &mut |event| events.push(event),
        )
        .unwrap();
        assert_eq!(report.cache_disposition, disposition);
        assert_eq!(
            events,
            vec![crate::QueryProgressEvent::CacheRefresh {
                component: "subnet catalog".to_string(),
                path: path.clone(),
                source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
            }]
        );
    }
    let source_calls = source.call_count();
    request.read_policy = CatalogReadPolicy::CacheOnly;
    events.clear();
    build_subnet_catalog_list_report_with_source_and_progress(&request, &source, &mut |event| {
        events.push(event);
    })
    .unwrap();
    request.read_policy = list_request(&root).read_policy;
    build_subnet_catalog_list_report_with_source_and_progress(&request, &source, &mut |event| {
        events.push(event);
    })
    .unwrap();
    assert_eq!(events, Vec::<crate::QueryProgressEvent>::new());
    assert_eq!(source.call_count(), source_calls);
    fs::remove_file(&path).unwrap();
    request.read_policy = CatalogReadPolicy::CacheOnly;
    assert!(
        build_subnet_catalog_list_report_with_source_and_progress(
            &request,
            &source,
            &mut |event| events.push(event)
        )
        .is_err()
    );
    assert_eq!(events, Vec::<crate::QueryProgressEvent>::new());
    assert_eq!(source.call_count(), source_calls);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn catalog_lock_refusal_does_not_announce_a_source_call() {
    let root = temp_dir("ic-query-catalog-progress-lock");
    let request = list_request(&root);
    let refresh = refresh_request(&root);
    let lock = subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK);
    write_refresh_lock_for_test(&lock, &refresh, request.now_unix_secs * 1_000);
    let source = FixtureRefreshSource::ok(fixture_catalog());
    let mut events = Vec::new();
    assert!(
        build_subnet_catalog_list_report_with_source_and_progress(
            &request,
            &source,
            &mut |event| events.push(event)
        )
        .is_err()
    );
    assert_eq!(events, Vec::<crate::QueryProgressEvent>::new());
    assert_eq!(source.call_count(), 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn list_report_loads_cached_catalog_and_caps_ranges() {
    let root = temp_dir("ic-query-subnet-list");
    write_catalog(&root, fixture_catalog());
    let request = list_request(&root);

    let report = build_subnet_catalog_list_report(&request).expect("list report");
    let text = subnet_catalog_list_report_text(&report);

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.subnets.len(), 2);
    assert_eq!(report.cache_disposition, CacheDisposition::CacheHit);
    assert_eq!(report.assurance, CatalogAssurance::UncertifiedQuery);
    assert_eq!(report.subnets[0].range_count, 2);
    assert_eq!(report.subnets[0].ranges_shown, 1);
    assert!(text.contains("SUBNET"));
    assert!(text.contains("SPEC"));
    assert!(!text.contains("SPECIALIZATION"));
    for subnet in &report.subnets {
        let compact_principal = subnet.subnet_principal.chars().take(5).collect::<String>();
        assert!(text.contains(&compact_principal));
        assert!(!text.contains(&subnet.subnet_principal));
    }
    assert!(!text.contains("FETCHED_AT"));
    assert!(text.contains("showing 1 of 2 ranges"));
    assert!(text.contains("\n\nSUBNET"));
}

#[test]
fn list_report_refreshes_missing_catalog() {
    let root = temp_dir("ic-query-subnet-list-refresh");
    let mut catalog = fixture_catalog();
    catalog.provenance.registry_version = 987_654;
    catalog.canonicalize_and_seal().expect("reseal fixture");
    let source = FixtureRefreshSource::ok(catalog);
    let request = list_request(&root);

    let report =
        build_subnet_catalog_list_report_with_source(&request, &source).expect("list report");
    let cached =
        load_cached_subnet_catalog(&cache_only_load_request(&root)).expect("cached catalog");

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.registry_version, 987_654);
    assert_eq!(report.cache_disposition, CacheDisposition::RefreshedMissing);
    assert_eq!(cached.catalog.provenance().registry_version, 987_654);
}

#[test]
fn list_report_refreshes_invalid_catalog_but_cache_only_remains_strict() {
    let root = temp_dir("ic-query-subnet-list-invalid-refresh");
    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    crate::cache_file::write_managed_text_atomically(&root, &path, "not-json")
        .expect("write invalid catalog");
    let request = list_request(&root);

    let error = load_cached_subnet_catalog(&cache_only_load_request(&root))
        .expect_err("cache-only load is strict");
    assert!(matches!(
        error,
        SubnetCatalogHostError::Catalog(CatalogError::Json(_))
    ));

    let mut catalog = fixture_catalog();
    catalog.provenance.registry_version = 987_654;
    catalog.canonicalize_and_seal().expect("reseal fixture");
    let report =
        build_subnet_catalog_list_report_with_source(&request, &FixtureRefreshSource::ok(catalog))
            .expect("invalid catalog refreshes");

    assert_eq!(report.registry_version, 987_654);
    assert_eq!(report.cache_disposition, CacheDisposition::RefreshedInvalid);
    assert_ne!(
        fs::read_to_string(path).expect("refreshed catalog"),
        "not-json"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_invalid_catalog_refresh_preserves_original_file() {
    let root = temp_dir("ic-query-subnet-list-invalid-refresh-failure");
    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    crate::cache_file::write_managed_text_atomically(&root, &path, "not-json")
        .expect("write invalid catalog");

    let error = build_subnet_catalog_list_report_with_source(
        &list_request(&root),
        &FixtureRefreshSource::err(),
    )
    .expect_err("failed refresh remains visible");

    assert!(matches!(
        error,
        SubnetCatalogHostError::Catalog(CatalogError::EmptySubnets)
    ));
    assert_eq!(
        fs::read_to_string(path).expect("preserved invalid catalog"),
        "not-json"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn list_report_verbose_text_keeps_full_metadata() {
    let root = temp_dir("ic-query-subnet-list-verbose");
    write_catalog(&root, fixture_catalog());
    let request = list_request(&root);

    let report = build_subnet_catalog_list_report(&request).expect("list report");
    let text = subnet_catalog_list_report_verbose_text(&report);

    let _ = fs::remove_dir_all(root);
    assert!(text.contains("catalog_path:"));
    assert!(text.contains("SPECIALIZATION"));
    assert!(text.contains("FETCHED_AT"));
    assert!(text.contains(SUBNET_A));
    assert!(text.contains("\n\nSUBNET"));
}
