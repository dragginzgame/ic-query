use super::{fixtures::*, *};
use crate::{CacheFileError, HostCacheError};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn catalog_path_lives_under_cache_root() {
    let root = PathBuf::from("/tmp/ic-query-cache");

    let path = subnet_catalog_path(&root, MAINNET_NETWORK);

    assert_eq!(
        path,
        PathBuf::from("/tmp/ic-query-cache/nns/ic/subnet-catalog/catalog.json")
    );
    assert!(!path.display().to_string().contains("/deployments/"));
    assert!(!path.display().to_string().contains("/fleets/"));
}

#[test]
fn load_cached_catalog_rejects_non_mainnet_network() {
    let root = temp_dir("ic-query-subnet-network");
    let request = SubnetCatalogLoadRequest::cache_only(
        SubnetCatalogCacheRequest {
            cache_root: root.clone(),
            network: "local".to_string(),
        },
        1_780_531_300,
    );

    let err = load_cached_subnet_catalog(&request).expect_err("local rejected");

    let _ = fs::remove_dir_all(root);
    assert!(matches!(
        err,
        SubnetCatalogHostError::UnsupportedNetwork { .. }
    ));
}

#[test]
fn missing_catalog_error_explains_cached_only_slice() {
    let root = temp_dir("ic-query-subnet-missing");
    let request = SubnetCatalogLoadRequest::cache_only(
        SubnetCatalogCacheRequest {
            cache_root: root.clone(),
            network: MAINNET_NETWORK.to_string(),
        },
        1_780_531_300,
    );

    let err = load_cached_subnet_catalog(&request).expect_err("cache missing");
    let _ = fs::remove_dir_all(root);
    assert_eq!(err.code(), SubnetCatalogErrorCode::MissingCatalog);
    assert_eq!(err.category(), SubnetCatalogErrorCategory::Missing);
    assert_eq!(
        err.remediation(),
        Some(SubnetCatalogRemediation::RefreshCatalog)
    );
}

#[test]
fn detailed_cache_failures_distinguish_absence_and_rejection() {
    let root = temp_dir("ic-query-subnet-detailed-cache-stages");
    let request = cache_only_load_request(&root);

    let missing = load_cached_subnet_catalog_detailed(&request).expect_err("cache missing");
    assert_eq!(missing.stage, SubnetCatalogLoadStage::CacheAbsence);
    assert_eq!(
        missing.cache_disposition,
        SubnetCatalogFailureCacheDisposition::CacheMissing
    );
    assert_eq!(missing.registry_version, None);

    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    crate::cache_file::write_managed_text_atomically(&root, &path, "not-json")
        .expect("invalid cache");
    let rejected = load_cached_subnet_catalog_detailed(&request).expect_err("cache rejected");

    let _ = fs::remove_dir_all(root);
    assert_eq!(rejected.stage, SubnetCatalogLoadStage::CacheRejection);
    assert_eq!(
        rejected.cache_disposition,
        SubnetCatalogFailureCacheDisposition::CacheRejected
    );
    assert_eq!(rejected.registry_version, None);
    assert!(matches!(
        rejected.source,
        SubnetCatalogHostError::Catalog(CatalogError::Json(_))
    ));
}

#[test]
fn detailed_refresh_failure_retains_request_version_subject_and_unknown_retryability() {
    let root = temp_dir("ic-query-subnet-detailed-refresh-failure");
    let endpoint = DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT;
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::RefreshMissing {
            source: CatalogSourceSelection::uncertified_query(endpoint),
        });
    let subject = SubnetCatalogSubject::RegistryRecord(SubnetCatalogRegistryRecordSubject {
        kind: SubnetCatalogRegistryRecordKind::SubnetList,
        key: Some(crate::ic_registry::SUBNET_LIST_KEY.to_string()),
        subnet: None,
    });

    let failure = load_subnet_catalog_detailed_with_source(
        &request,
        &DetailedFailureSource::new(Some(881_337), Some(subject.clone()), "SubnetListRecord"),
    )
    .expect_err("refresh fails");

    assert_eq!(failure.stage, SubnetCatalogLoadStage::RefreshFailed);
    assert_eq!(
        failure.cache_disposition,
        SubnetCatalogFailureCacheDisposition::RefreshFailed(SubnetCatalogRefreshTrigger::Missing)
    );
    assert_eq!(failure.registry_version, Some(881_337));
    assert_eq!(failure.subject, Some(subject));
    assert_eq!(failure.request.network, MAINNET_NETWORK);
    assert_eq!(
        failure.request.minimum_assurance,
        CatalogAssurance::UncertifiedQuery
    );
    assert_eq!(
        failure.request.source,
        Some(CatalogSourceSelection::uncertified_query(endpoint))
    );
    assert_eq!(failure.code, SubnetCatalogErrorCode::RegistryRefresh);
    assert_eq!(failure.category, SubnetCatalogErrorCategory::Network);
    assert_eq!(
        failure.retryability,
        SubnetCatalogRetryability::Unknown(SubnetCatalogUnknownRetryReason::RegistryResponse)
    );
    assert!(matches!(
        failure.source,
        SubnetCatalogHostError::RegistryRefresh(
            crate::ic_registry::RegistryFetchError::ProtobufDecode {
                message: "SubnetListRecord",
                ..
            }
        )
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn detailed_source_failures_before_and_after_version_acquisition_are_truthful() {
    let cases = [
        (
            None,
            SubnetCatalogSubject::RegistryRecord(SubnetCatalogRegistryRecordSubject {
                kind: SubnetCatalogRegistryRecordKind::LatestVersion,
                key: None,
                subnet: None,
            }),
            "RegistryGetLatestVersionResponse",
        ),
        (
            Some(445_566),
            SubnetCatalogSubject::RegistryRecord(SubnetCatalogRegistryRecordSubject {
                kind: SubnetCatalogRegistryRecordKind::SubnetList,
                key: Some(crate::ic_registry::SUBNET_LIST_KEY.to_string()),
                subnet: None,
            }),
            "SubnetListRecord",
        ),
        (
            Some(445_566),
            SubnetCatalogSubject::RegistryRecord(SubnetCatalogRegistryRecordSubject {
                kind: SubnetCatalogRegistryRecordKind::RoutingTable,
                key: Some(crate::ic_registry::ROUTING_TABLE_KEY.to_string()),
                subnet: None,
            }),
            "RoutingTable",
        ),
        (
            Some(445_566),
            SubnetCatalogSubject::RegistryRecord(SubnetCatalogRegistryRecordSubject {
                kind: SubnetCatalogRegistryRecordKind::SubnetRecord,
                key: Some(crate::ic_registry::subnet_record_key(SUBNET_A)),
                subnet: Some(candid::Principal::from_text(SUBNET_A).expect("subnet")),
            }),
            "SubnetRecord",
        ),
    ];

    for (index, (registry_version, subject, message)) in cases.into_iter().enumerate() {
        let root = temp_dir(&format!("ic-query-subnet-version-failure-{index}"));
        let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
            .with_policy(CatalogReadPolicy::ForceRefresh {
                source: CatalogSourceSelection::uncertified_query(
                    DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
                ),
            });
        let failure = load_subnet_catalog_detailed_with_source(
            &request,
            &DetailedFailureSource::new(registry_version, Some(subject.clone()), message),
        )
        .expect_err("fixture fails");

        assert_eq!(failure.registry_version, registry_version);
        assert_eq!(failure.subject, Some(subject));
        assert_eq!(failure.stage, SubnetCatalogLoadStage::RefreshFailed);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn simple_load_api_returns_the_original_observable_source_error() {
    let root = temp_dir("ic-query-subnet-simple-source-error");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::ForceRefresh {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
        });

    let failure = load_subnet_catalog_with_source(
        &request,
        &DetailedFailureSource::new(Some(77), None, "RoutingTable"),
    )
    .expect_err("simple API fails");

    let _ = fs::remove_dir_all(root);
    assert!(matches!(
        failure,
        SubnetCatalogHostError::RegistryRefresh(
            crate::ic_registry::RegistryFetchError::ProtobufDecode {
                message: "RoutingTable",
                ..
            }
        )
    ));
}

#[test]
fn cache_only_policy_never_invokes_the_supplied_source() {
    let root = temp_dir("ic-query-subnet-cache-only-source");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300);

    let error = load_subnet_catalog_with_source(&request, &FixtureRefreshSource::err())
        .expect_err("missing cache remains a cache-only failure");

    let _ = fs::remove_dir_all(root);
    assert!(matches!(
        error,
        SubnetCatalogHostError::MissingCatalog { .. }
    ));
}

#[test]
fn every_simple_load_entry_point_preserves_the_missing_catalog_error() {
    let root = temp_dir("ic-query-subnet-simple-load-wrappers");
    let request = cache_only_load_request(&root);
    let source = FixtureRefreshSource::err();

    let cached = load_cached_subnet_catalog(&request).expect_err("cached load");
    let sync = load_subnet_catalog(&request).expect_err("sync load");
    let sync_source =
        load_subnet_catalog_with_source(&request, &source).expect_err("sync source load");
    let async_load =
        futures::executor::block_on(load_subnet_catalog_async(&request)).expect_err("async load");
    let async_source =
        futures::executor::block_on(load_subnet_catalog_with_source_async(&request, &source))
            .expect_err("async source load");

    let _ = fs::remove_dir_all(root);
    for failure in [cached, sync, sync_source, async_load, async_source] {
        assert!(matches!(
            failure,
            SubnetCatalogHostError::MissingCatalog { .. }
        ));
    }
}

#[test]
fn caller_minimum_assurance_rejects_weaker_cached_evidence() {
    let root = temp_dir("ic-query-subnet-minimum-assurance");
    write_catalog(&root, fixture_catalog());
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_minimum_assurance(CatalogAssurance::MultiEndpointAgreement);

    let error = load_cached_subnet_catalog(&request).expect_err("weak cache rejected");

    let _ = fs::remove_dir_all(root);
    assert_eq!(error.code(), SubnetCatalogErrorCode::InsufficientAssurance);
    assert_eq!(error.category(), SubnetCatalogErrorCategory::Authority);
    assert!(matches!(
        error,
        SubnetCatalogHostError::InsufficientAssurance {
            required: CatalogAssurance::MultiEndpointAgreement,
            actual: CatalogAssurance::UncertifiedQuery,
        }
    ));
}

#[test]
fn insufficient_refresh_selection_fails_before_collection() {
    let root = temp_dir("ic-query-subnet-refresh-minimum-assurance");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_minimum_assurance(CatalogAssurance::MultiEndpointAgreement)
        .with_policy(CatalogReadPolicy::ForceRefresh {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
        });

    let error = load_subnet_catalog_with_source(&request, &FixtureRefreshSource::err())
        .expect_err("weak source selection rejected before collection");

    let _ = fs::remove_dir_all(root);
    assert!(matches!(
        error,
        SubnetCatalogHostError::InsufficientAssurance {
            required: CatalogAssurance::MultiEndpointAgreement,
            actual: CatalogAssurance::UncertifiedQuery,
        }
    ));
}

#[test]
fn missing_only_policy_does_not_repair_invalid_content() {
    let root = temp_dir("ic-query-subnet-missing-only-invalid");
    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    crate::cache_file::write_managed_text_atomically(&root, &path, "not-json")
        .expect("invalid cache");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::RefreshMissing {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
        });

    let error =
        load_subnet_catalog_with_source(&request, &FixtureRefreshSource::ok(fixture_catalog()))
            .expect_err("missing-only policy keeps invalid content visible");

    assert!(matches!(
        error,
        SubnetCatalogHostError::Catalog(CatalogError::Json(_))
    ));
    assert_eq!(fs::read_to_string(path).expect("cache remains"), "not-json");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_only_policy_reports_missing_refresh_disposition() {
    let root = temp_dir("ic-query-subnet-missing-only");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::RefreshMissing {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
        });

    let outcome =
        load_subnet_catalog_with_source(&request, &FixtureRefreshSource::ok(fixture_catalog()))
            .expect("missing content refreshes");

    let _ = fs::remove_dir_all(root);
    assert_eq!(outcome.disposition, CacheDisposition::RefreshedMissing);
}

#[test]
fn async_load_policy_refreshes_without_an_internal_runtime_adapter() {
    let root = temp_dir("ic-query-subnet-async-load");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::RefreshMissing {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
        });

    let outcome = futures::executor::block_on(load_subnet_catalog_with_source_async(
        &request,
        &FixtureRefreshSource::ok(fixture_catalog()),
    ))
    .expect("async load refreshes missing content");

    assert_eq!(outcome.disposition, CacheDisposition::RefreshedMissing);
    assert_eq!(outcome.catalog.provenance().registry_query_call_count, 5);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stale_and_forced_policies_report_exact_dispositions() {
    let root = temp_dir("ic-query-subnet-stale-disposition");
    let mut old = fixture_catalog();
    old.provenance.fetched_at = "1970-01-01T00:00:00Z".to_string();
    old.canonicalize_and_seal().expect("seal old fixture");
    write_catalog(&root, old);

    let mut replacement = fixture_catalog();
    replacement.provenance.registry_version = 987_654;
    replacement.provenance.fetched_at = "2026-06-04T00:01:40Z".to_string();
    replacement
        .canonicalize_and_seal()
        .expect("seal replacement fixture");
    let stale_request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::RefreshMissingInvalidOrOlderThan {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
            max_age_seconds: 60,
        });

    let stale = load_subnet_catalog_with_source(
        &stale_request,
        &FixtureRefreshSource::ok(replacement.clone()),
    )
    .expect("stale content refreshes");
    assert_eq!(stale.disposition, CacheDisposition::RefreshedStale);
    assert_eq!(stale.catalog.provenance().registry_version, 987_654);

    let fresh = load_subnet_catalog_with_source(&stale_request, &FixtureRefreshSource::err())
        .expect("fresh content does not invoke source");
    assert_eq!(fresh.disposition, CacheDisposition::CacheHit);

    replacement.provenance.registry_version = 987_655;
    replacement
        .canonicalize_and_seal()
        .expect("reseal forced fixture");
    let forced_request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::ForceRefresh {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
        });
    let forced =
        load_subnet_catalog_with_source(&forced_request, &FixtureRefreshSource::ok(replacement))
            .expect("forced content refreshes");

    let _ = fs::remove_dir_all(root);
    assert_eq!(forced.disposition, CacheDisposition::ForcedRefresh);
    assert_eq!(forced.catalog.provenance().registry_version, 987_655);
}

#[cfg(unix)]
#[test]
fn catalog_load_rejects_symlinked_managed_parent_without_refreshing() {
    let root = temp_dir("ic-query-subnet-symlink-parent");
    let outside = temp_dir("ic-query-subnet-symlink-outside");
    crate::cache_file::write_managed_text_atomically(&root, &root.join("seed"), "seed")
        .expect("create confined root");
    fs::create_dir_all(&outside).expect("create outside");
    symlink(&outside, root.join("nns")).expect("link managed parent");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::RefreshMissingOrInvalid {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
        });

    let error = load_subnet_catalog_with_source(&request, &FixtureRefreshSource::err())
        .expect_err("symlink is not recoverable invalid content");

    assert_eq!(error.category(), SubnetCatalogErrorCategory::Confinement);
    assert!(matches!(
        error,
        SubnetCatalogHostError::Cache(HostCacheError::Operation {
            source: CacheFileError::Confinement { .. },
            ..
        })
    ));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}

#[cfg(unix)]
#[test]
fn catalog_load_rejects_unsafe_managed_file_mode() {
    let root = temp_dir("ic-query-subnet-unsafe-mode");
    write_catalog(&root, fixture_catalog());
    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("widen cache mode");

    let error = load_cached_subnet_catalog(&cache_only_load_request(&root))
        .expect_err("unsafe mode rejected");

    assert_eq!(error.category(), SubnetCatalogErrorCategory::Confinement);
    assert!(matches!(
        error,
        SubnetCatalogHostError::Cache(HostCacheError::Operation {
            source: CacheFileError::UnsafeManagedPermissions {
                actual_mode: 0o644,
                ..
            },
            ..
        })
    ));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("restore cache mode");
    let _ = fs::remove_dir_all(root);
}
