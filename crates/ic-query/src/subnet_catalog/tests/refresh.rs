use super::{fixtures::*, *};
use crate::cache_file::{CacheFileError, HostCacheError};
use crate::nns::{LiveNnsSource, NnsSourceRequest};
use std::{
    future::Future,
    task::{Context, Poll},
};

struct PendingSource;

impl SubnetCatalogSource for PendingSource {
    fn fetch_catalog<'a>(
        &'a self,
        _request: &'a NnsSourceRequest,
    ) -> SubnetCatalogSourceFuture<'a> {
        Box::pin(std::future::pending())
    }
}

struct WrongEndpointSource;

impl SubnetCatalogSource for WrongEndpointSource {
    fn fetch_catalog<'a>(
        &'a self,
        _request: &'a NnsSourceRequest,
    ) -> SubnetCatalogSourceFuture<'a> {
        Box::pin(async { Ok(fixture_catalog()) })
    }
}

#[test]
fn live_catalog_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = crate::runtime::block_on_current_thread(LiveNnsSource.fetch_catalog(&request))
        .expect("test runtime")
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        SubnetCatalogHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn public_async_catalog_fetch_rejects_non_mainnet_before_agent_construction() {
    let request = NnsSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = crate::runtime::block_on_current_thread(fetch_subnet_catalog_async(&request))
        .expect("test runtime")
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        SubnetCatalogHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn refresh_writes_catalog_atomically_and_removes_lock() {
    let root = temp_dir("ic-query-subnet-refresh");
    let mut catalog = fixture_catalog();
    catalog.provenance.registry_version = 987_654;
    catalog.provenance.fetched_at = "1970-01-01T00:00:00Z".to_string();
    catalog.provenance.source_endpoints = vec![DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string()];
    catalog.canonicalize_and_seal().expect("reseal fixture");
    let source = FixtureRefreshSource::ok(catalog);
    let request = refresh_request(&root);

    let report = refresh_subnet_catalog_with_source(&request, &source).expect("refresh catalog");
    let cached =
        load_cached_subnet_catalog(&cache_only_load_request(&root)).expect("cached catalog");
    let lock_path = PathBuf::from(&report.refresh_lock_path);

    let _ = fs::remove_dir_all(root);
    assert!(report.wrote_catalog);
    assert!(!report.replaced_existing_catalog);
    assert_eq!(report.registry_version, 987_654);
    assert_eq!(report.assurance, CatalogAssurance::UncertifiedQuery);
    assert!(report.agreement_digest.is_none());
    assert_eq!(report.registry_query_call_count, 5);
    assert_eq!(
        report.source_endpoints,
        vec![DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string()]
    );
    assert_eq!(report.catalog_digest, cached.catalog.raw().catalog_digest);
    assert_eq!(cached.catalog.provenance().registry_version, 987_654);
    assert!(!lock_path.exists());
}

#[test]
fn async_multi_endpoint_refresh_publishes_matching_agreement_evidence() {
    let root = temp_dir("ic-query-subnet-refresh-agreement");
    let alpha = "https://alpha.example";
    let beta = "https://beta.example";
    let source = AgreementFixtureSource::new(AgreementFixtureMode::Matching, beta);
    let mut request = refresh_request(&root);
    request.source =
        CatalogSourceSelection::multi_endpoint_agreement(vec![beta.to_string(), alpha.to_string()]);

    let report =
        futures::executor::block_on(refresh_subnet_catalog_with_source_async(&request, &source))
            .expect("matching endpoints refresh");
    let cached =
        load_cached_subnet_catalog(&cache_only_load_request(&root)).expect("agreement cache");

    assert_eq!(source.call_count(), 2);
    assert_eq!(report.assurance, CatalogAssurance::MultiEndpointAgreement);
    assert_eq!(
        report.source_endpoints,
        vec![alpha.to_string(), beta.to_string()]
    );
    assert_eq!(report.registry_query_call_count, 10);
    assert_eq!(report.registry_records.len(), 8);
    assert!(report.registry_records.iter().all(|evidence| {
        evidence.requested_registry_version == report.registry_version
            && report.source_endpoints.contains(&evidence.source_endpoint)
    }));
    assert_eq!(
        report.agreement_digest,
        cached.catalog.provenance().agreement_digest
    );
    assert!(report.agreement_digest.is_some());
    assert_eq!(
        cached.catalog.provenance().assurance,
        CatalogAssurance::MultiEndpointAgreement
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn detailed_agreement_mismatch_retains_every_completed_endpoint_record() {
    let root = temp_dir("ic-query-subnet-detailed-agreement-mismatch");
    let alpha = "https://alpha.example";
    let beta = "https://beta.example";
    let source = AgreementFixtureSource::new(AgreementFixtureMode::PayloadMismatch, beta);
    let request = SubnetCatalogLoadRequest::cache_only(cache_request(&root), 1_780_531_300)
        .with_policy(CatalogReadPolicy::ForceRefresh {
            source: CatalogSourceSelection::multi_endpoint_agreement(vec![
                alpha.to_string(),
                beta.to_string(),
            ]),
        });

    let failure = futures::executor::block_on(load_subnet_catalog_detailed_with_source_async(
        &request, &source,
    ))
    .expect_err("mismatched endpoint evidence");

    assert_eq!(failure.stage, SubnetCatalogLoadStage::RefreshFailed);
    assert_eq!(failure.registry_records.len(), 8);
    assert!(
        failure.registry_records.iter().all(|evidence| {
            evidence.source_endpoint == alpha || evidence.source_endpoint == beta
        })
    );
    assert!(!subnet_catalog_path(&root, MAINNET_NETWORK).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn agreement_rejects_version_and_payload_mismatch_without_publishing() {
    for (name, mode) in [
        ("version", AgreementFixtureMode::VersionMismatch),
        ("payload", AgreementFixtureMode::PayloadMismatch),
    ] {
        let root = temp_dir(&format!("ic-query-subnet-agreement-{name}"));
        write_catalog(&root, fixture_catalog());
        let beta = "https://beta.example";
        let source = AgreementFixtureSource::new(mode, beta);
        let mut request = refresh_request(&root);
        request.source = CatalogSourceSelection::multi_endpoint_agreement(vec![
            "https://alpha.example".to_string(),
            beta.to_string(),
        ]);

        let error = futures::executor::block_on(refresh_subnet_catalog_with_source_async(
            &request, &source,
        ))
        .expect_err("mismatched endpoint evidence");
        let cached =
            load_cached_subnet_catalog(&cache_only_load_request(&root)).expect("original cache");

        assert!(matches!(
            error,
            SubnetCatalogHostError::AgreementMismatch { endpoint, .. } if endpoint == beta
        ));
        assert_eq!(cached.catalog.provenance().registry_version, 123_456);
        assert!(!subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK).exists());
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn invalid_agreement_selection_is_rejected_before_source_or_cache_io() {
    for endpoints in [
        vec![
            "https://same.example".to_string(),
            "https://same.example:8443".to_string(),
        ],
        vec!["https://only.example".to_string()],
        vec![
            "https://one.example".to_string(),
            "https://two.example".to_string(),
            "https://three.example".to_string(),
            "https://four.example".to_string(),
        ],
    ] {
        let root = temp_dir("ic-query-subnet-invalid-agreement");
        let source =
            AgreementFixtureSource::new(AgreementFixtureMode::Matching, "https://none.example");
        let mut request = refresh_request(&root);
        request.source = CatalogSourceSelection::multi_endpoint_agreement(endpoints);

        let error = futures::executor::block_on(refresh_subnet_catalog_with_source_async(
            &request, &source,
        ))
        .expect_err("invalid selection");

        assert!(matches!(
            error,
            SubnetCatalogHostError::InvalidSourceSelection { .. }
        ));
        assert_eq!(source.call_count(), 0);
        assert!(!subnet_catalog_path(&root, MAINNET_NETWORK).exists());
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn agreement_endpoint_failure_keeps_exact_endpoint_context() {
    let root = temp_dir("ic-query-subnet-agreement-endpoint-error");
    let beta = "https://beta.example";
    let source = AgreementFixtureSource::new(AgreementFixtureMode::EndpointFailure, beta);
    let mut request = refresh_request(&root);
    request.source = CatalogSourceSelection::multi_endpoint_agreement(vec![
        "https://alpha.example".to_string(),
        beta.to_string(),
    ]);

    let error =
        futures::executor::block_on(refresh_subnet_catalog_with_source_async(&request, &source))
            .expect_err("endpoint fails");

    assert!(matches!(
        error,
        SubnetCatalogHostError::AgreementEndpoint { endpoint, .. } if endpoint == beta
    ));
    assert_eq!(source.call_count(), 2);
    assert!(!subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn cancelled_async_refresh_drops_its_owned_lock_without_publishing() {
    let root = temp_dir("ic-query-subnet-refresh-cancelled");
    let request = refresh_request(&root);
    let lock_path = subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK);
    let mut future = Box::pin(refresh_subnet_catalog_with_source_async(
        &request,
        &PendingSource,
    ));
    let waker = futures::task::noop_waker();
    let mut context = Context::from_waker(&waker);

    assert!(matches!(future.as_mut().poll(&mut context), Poll::Pending));
    assert!(lock_path.exists());
    drop(future);

    assert!(!lock_path.exists());
    assert!(!subnet_catalog_path(&root, MAINNET_NETWORK).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn custom_source_must_echo_the_exact_requested_endpoint() {
    let root = temp_dir("ic-query-subnet-source-endpoint-mismatch");
    let requested = "https://other.example";
    let mut request = refresh_request(&root);
    request.source = CatalogSourceSelection::uncertified_query(requested);

    let error = futures::executor::block_on(refresh_subnet_catalog_with_source_async(
        &request,
        &WrongEndpointSource,
    ))
    .expect_err("source endpoint must match request");

    assert!(matches!(
        error,
        SubnetCatalogHostError::SourceEvidenceMismatch {
            requested: actual_request,
            actual_endpoints,
            ..
        } if actual_request == requested
            && actual_endpoints == [DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT]
    ));
    assert!(!subnet_catalog_path(&root, MAINNET_NETWORK).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn refresh_dry_run_writes_output_without_replacing_cache() {
    let root = temp_dir("ic-query-subnet-refresh-dry-run");
    let mut catalog = fixture_catalog();
    catalog.provenance.fetched_at = "1970-01-01T00:00:00Z".to_string();
    catalog.provenance.source_endpoints = vec![DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string()];
    catalog.canonicalize_and_seal().expect("reseal fixture");
    let output_path = root.join("catalog-export.json");
    let source = FixtureRefreshSource::ok(catalog);
    let mut request = refresh_request(&root);
    request.dry_run = true;
    request.output_path = Some(output_path.clone());

    let report = refresh_subnet_catalog_with_source(&request, &source).expect("dry-run");

    assert!(!report.wrote_catalog);
    assert!(!subnet_catalog_path(&request.cache.cache_root, MAINNET_NETWORK).exists());
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
    let cached =
        load_cached_subnet_catalog(&cache_only_load_request(&root)).expect("cached catalog");
    let lock_path = subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK);

    assert!(matches!(
        err,
        SubnetCatalogHostError::Catalog(CatalogError::EmptySubnets)
    ));
    assert_eq!(cached.catalog.provenance().registry_version, 123_456);
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
        SubnetCatalogHostError::Cache(HostCacheError::Operation {
            source: CacheFileError::RefreshAlreadyInProgress { .. },
            ..
        })
    ));
}

#[test]
fn refresh_rejects_stale_lock_without_removing_it() {
    let root = temp_dir("ic-query-subnet-refresh-stale-lock");
    let mut catalog = fixture_catalog();
    catalog.provenance.fetched_at = "1970-01-01T00:00:00Z".to_string();
    catalog.provenance.source_endpoints = vec![DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string()];
    catalog.canonicalize_and_seal().expect("reseal fixture");
    let source = FixtureRefreshSource::ok(catalog);
    let request = refresh_request(&root);
    let lock_path = subnet_catalog_refresh_lock_path(&root, MAINNET_NETWORK);
    let stale_started_at = (request.now_unix_secs - request.lock_stale_after_seconds - 1) * 1_000;
    write_refresh_lock_for_test(&lock_path, &request, stale_started_at);

    let err = refresh_subnet_catalog_with_source(&request, &source)
        .expect_err("stale lock requires manual cleanup");

    assert!(matches!(
        err,
        SubnetCatalogHostError::Cache(HostCacheError::Operation {
            source: CacheFileError::StaleRefreshLock { .. },
            ..
        })
    ));
    assert!(lock_path.exists());
    let _ = fs::remove_dir_all(root);
}
