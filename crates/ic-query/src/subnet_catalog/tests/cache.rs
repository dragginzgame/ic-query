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
fn missing_only_policy_does_not_repair_invalid_content() {
    let root = temp_dir("ic-query-subnet-missing-only-invalid");
    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    crate::cache_file::write_managed_text_atomically(&root, &path, "not-json")
        .expect("invalid cache");
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::RefreshMissing {
            source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
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
            source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        });

    let outcome =
        load_subnet_catalog_with_source(&request, &FixtureRefreshSource::ok(fixture_catalog()))
            .expect("missing content refreshes");

    let _ = fs::remove_dir_all(root);
    assert_eq!(outcome.disposition, CacheDisposition::RefreshedMissing);
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
            source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
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
            source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
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
            source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
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
