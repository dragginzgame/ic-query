//! Module: nns::registry::replay::tests::cache
//!
//! Responsibility: certified Subnet-catalog cache publication and recovery tests.
//! Does not own: production replay behavior or shared protocol fixtures.
//! Boundary: exercises the corresponding replay subsystem through fixture evidence.

use super::*;

#[test]
fn certified_catalog_cache_round_trip_requalifies_freshness_from_the_archive() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-cache-round-trip");
    let archive = complete_catalog_archive(&root);
    let cache_directory = root.join("nns/ic/registry-certified-catalog-v1");
    let location = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 100_000);
    let publication =
        NnsCertifiedSubnetCatalogLoadRequest::force_publication(location.clone(), 300);
    let initial = certified_catalog_projection_request(
        NOW,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );

    let published = load_nns_certified_subnet_catalog(&archive, &initial, &publication)
        .expect("publish archive-bound certified catalog cache");
    let cache_path = nns_certified_subnet_catalog_cache_path(&cache_directory);
    let envelope: NnsCertifiedSubnetCatalogCacheEnvelope =
        serde_json::from_slice(&fs::read(&cache_path).expect("read published cache"))
            .expect("decode published envelope");

    assert_eq!(
        envelope.schema_version,
        NNS_CERTIFIED_SUBNET_CATALOG_CACHE_SCHEMA_VERSION
    );
    assert_eq!(
        envelope.catalog.provenance.assurance,
        CatalogAssurance::Certified
    );
    assert_eq!(envelope.catalog.provenance.registry_version, 1);
    assert_eq!(envelope.archive_manifest_sha256.len(), 64);
    assert_eq!(
        envelope.catalog.catalog_digest,
        published.authority().catalog().raw().catalog_digest
    );
    assert_eq!(published.path(), cache_path);
    assert_eq!(
        published.disposition(),
        NnsCertifiedSubnetCatalogCacheDisposition::ForcedPublication
    );
    let published_authority = published.snapshot_authority();
    let published_evidence = published.cache_evidence();
    assert_eq!(published_evidence.registry_version, 1);
    assert_eq!(
        published_evidence.archive_manifest_sha256,
        envelope.archive_manifest_sha256
    );
    assert_eq!(
        published_evidence.root_key_digest,
        archive.manifest().root_key_digest
    );
    assert_eq!(
        published_evidence.cache_disposition,
        NnsCertifiedSubnetCatalogCacheDisposition::ForcedPublication
    );
    let serialized_evidence =
        serde_json::to_vec(&published_evidence).expect("serialize compact cache evidence");
    assert_eq!(
        serde_json::from_slice::<NnsCertifiedSubnetCatalogCacheEvidence>(&serialized_evidence)
            .expect("deserialize compact cache evidence"),
        published_evidence
    );
    assert!(cache_path.is_file());
    assert!(!nns_certified_subnet_catalog_cache_refresh_lock_path(&cache_directory).exists());

    let later = certified_catalog_projection_request(
        NOW + 30,
        30,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    let loaded = load_nns_certified_subnet_catalog(
        &archive,
        &later,
        &NnsCertifiedSubnetCatalogLoadRequest::cache_only(location),
    )
    .expect("reload against the same authenticated archive evidence");
    assert_eq!(
        loaded.authority().catalog(),
        published.authority().catalog()
    );
    assert_eq!(
        loaded.authority().freshness().certificate_age_nanos,
        30_000_000_000
    );
    assert_eq!(loaded.path(), cache_path);
    assert_eq!(
        loaded.disposition(),
        NnsCertifiedSubnetCatalogCacheDisposition::CacheHit
    );
    assert_eq!(published_authority, loaded.snapshot_authority());
    assert!(matches!(
        ValidatedSubnetCatalog::try_from_raw(
            envelope.catalog,
            &later.validation
        ),
        Err(CatalogError::UnsupportedAssurance { assurance }) if assurance == "certified"
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_catalog_cache_recovery_operations_are_explicit_and_observable() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-cache-recovery");
    let archive = complete_catalog_archive(&root);
    let cache_directory = root.join("nns/ic/registry-certified-catalog-v1");
    let location = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 100_000);
    let publish_missing =
        NnsCertifiedSubnetCatalogLoadRequest::publish_missing(location.clone(), 300);
    let publish_invalid =
        NnsCertifiedSubnetCatalogLoadRequest::publish_missing_or_invalid(location.clone(), 300);
    let cache_only = NnsCertifiedSubnetCatalogLoadRequest::cache_only(location);
    let projection = certified_catalog_projection_request(
        NOW,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );

    let missing = load_nns_certified_subnet_catalog(&archive, &projection, &publish_missing)
        .expect("publish explicitly missing cache");
    assert_eq!(
        missing.disposition(),
        NnsCertifiedSubnetCatalogCacheDisposition::PublishedMissing
    );
    assert_eq!(
        missing.cache_evidence().cache_disposition.as_str(),
        "published_missing"
    );
    let missing_authority = missing.snapshot_authority();

    let hit = load_nns_certified_subnet_catalog(&archive, &projection, &publish_missing)
        .expect("reuse valid cache");
    assert_eq!(
        hit.disposition(),
        NnsCertifiedSubnetCatalogCacheDisposition::CacheHit
    );
    assert_eq!(missing_authority, hit.snapshot_authority());

    let cache_path = nns_certified_subnet_catalog_cache_path(&cache_directory);
    let mut envelope: NnsCertifiedSubnetCatalogCacheEnvelope =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache"))
            .expect("decode cache envelope");
    envelope.catalog.provenance.registry_version += 1;
    let invalid = serde_json::to_vec(&envelope).expect("canonical invalid cache fixture");
    fs::write(&cache_path, &invalid).expect("write invalid cache fixture");

    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &projection, &publish_missing),
        Err(NnsCertifiedSubnetCatalogCacheError::ArchiveBindingMismatch { field: "catalog" })
    ));
    assert_eq!(
        fs::read(&cache_path).expect("read preserved invalid cache"),
        invalid
    );

    let repaired = load_nns_certified_subnet_catalog(&archive, &projection, &publish_invalid)
        .expect("explicitly replace recoverably invalid cache");
    assert_eq!(
        repaired.disposition(),
        NnsCertifiedSubnetCatalogCacheDisposition::PublishedInvalid
    );
    assert_ne!(fs::read(&cache_path).expect("read repaired cache"), invalid);
    assert_eq!(
        load_nns_certified_subnet_catalog(&archive, &projection, &cache_only)
            .expect("load repaired cache")
            .disposition(),
        NnsCertifiedSubnetCatalogCacheDisposition::CacheHit
    );

    fs::write(&cache_path, &invalid).expect("restore invalid cache fixture");
    let stale_projection = certified_catalog_projection_request(
        NOW + 2,
        1,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &stale_projection, &publish_invalid),
        Err(NnsCertifiedSubnetCatalogCacheError::Projection(
            NnsRegistrySubnetCatalogProjectionError::StaleArchiveCertificate { .. }
        ))
    ));
    assert_eq!(
        fs::read(&cache_path).expect("read invalid cache after stale projection"),
        invalid
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_catalog_cache_rejects_tampering_without_repairing_it() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-cache-tamper");
    let archive = complete_catalog_archive(&root);
    let cache_directory = root.join("nns/ic/registry-certified-catalog-v1");
    let location = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 100_000);
    let publication =
        NnsCertifiedSubnetCatalogLoadRequest::force_publication(location.clone(), 300);
    let cache_only = NnsCertifiedSubnetCatalogLoadRequest::cache_only(location);
    let projection = certified_catalog_projection_request(
        NOW,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    load_nns_certified_subnet_catalog(&archive, &projection, &publication)
        .expect("publish certified catalog cache");
    let cache_path = nns_certified_subnet_catalog_cache_path(&cache_directory);
    let envelope: NnsCertifiedSubnetCatalogCacheEnvelope =
        serde_json::from_slice(&fs::read(&cache_path).expect("read cache"))
            .expect("decode cache envelope");
    let mut mismatched = envelope.clone();
    mismatched.catalog.provenance.registry_version += 1;
    let tampered = serde_json::to_vec(&mismatched).expect("canonical tampered cache");
    fs::write(&cache_path, &tampered).expect("tamper cache fixture");

    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &projection, &cache_only),
        Err(NnsCertifiedSubnetCatalogCacheError::ArchiveBindingMismatch { field: "catalog" })
    ));
    assert_eq!(
        fs::read(&cache_path).expect("read rejected cache"),
        tampered
    );

    let noncanonical = serde_json::to_vec_pretty(&envelope).expect("pretty cache fixture");
    fs::write(&cache_path, &noncanonical).expect("write noncanonical cache fixture");
    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &projection, &cache_only),
        Err(NnsCertifiedSubnetCatalogCacheError::NonCanonicalEncoding { .. })
    ));

    let mut unsupported = envelope;
    unsupported.schema_version += 1;
    fs::write(
        &cache_path,
        serde_json::to_vec(&unsupported).expect("unsupported schema fixture"),
    )
    .expect("write unsupported schema fixture");
    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &projection, &cache_only),
        Err(
            NnsCertifiedSubnetCatalogCacheError::UnsupportedSchemaVersion {
                found: 2,
                supported: 1,
            }
        )
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_catalog_cache_failure_preserves_the_previous_atomic_snapshot() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-cache-preserve");
    let archive = complete_catalog_archive(&root);
    let cache_directory = root.join("nns/ic/registry-certified-catalog-v1");
    let location = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 100_000);
    let publication = NnsCertifiedSubnetCatalogLoadRequest::force_publication(location, 300);
    let projection = certified_catalog_projection_request(
        NOW,
        0,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );
    load_nns_certified_subnet_catalog(&archive, &projection, &publication)
        .expect("publish initial certified catalog cache");
    let cache_path = nns_certified_subnet_catalog_cache_path(&cache_directory);
    let previous = fs::read(&cache_path).expect("read initial cache");
    let limited_location = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 1);
    let limited = NnsCertifiedSubnetCatalogLoadRequest::force_publication(limited_location, 300);

    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &projection, &limited),
        Err(NnsCertifiedSubnetCatalogCacheError::CacheLimitExceeded { maximum: 1, .. })
    ));
    assert_eq!(
        fs::read(cache_path).expect("read preserved cache"),
        previous
    );

    let read_limited = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 1);
    assert!(matches!(
        load_nns_certified_subnet_catalog(
            &archive,
            &projection,
            &NnsCertifiedSubnetCatalogLoadRequest::cache_only(read_limited),
        ),
        Err(NnsCertifiedSubnetCatalogCacheError::CacheLimitExceeded { maximum: 1, .. })
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_catalog_cache_load_is_explicitly_cache_only() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-cache-missing");
    let archive = complete_catalog_archive(&root);
    let cache_directory = root.join("nns/ic/registry-certified-catalog-v1");
    let location = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 100_000);
    let stale_projection = certified_catalog_projection_request(
        NOW + 2,
        1,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );

    assert!(matches!(
        load_nns_certified_subnet_catalog(
            &archive,
            &stale_projection,
            &NnsCertifiedSubnetCatalogLoadRequest::cache_only(location),
        ),
        Err(NnsCertifiedSubnetCatalogCacheError::MissingCache { .. })
    ));
    assert!(!cache_directory.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn certified_catalog_cache_publication_qualifies_before_filesystem_mutation() {
    let root = crate::test_support::temp_dir("ic-query-certified-catalog-cache-preflight");
    let archive = complete_catalog_archive(&root);
    let cache_directory = root.join("nns/ic/registry-certified-catalog-v1");
    let location = NnsCertifiedSubnetCatalogCacheLocation::new(&root, &cache_directory, 100_000);
    let publication =
        NnsCertifiedSubnetCatalogLoadRequest::force_publication(location.clone(), 300);
    let publish_invalid =
        NnsCertifiedSubnetCatalogLoadRequest::publish_missing_or_invalid(location, 300);
    let stale_projection = certified_catalog_projection_request(
        NOW + 2,
        1,
        NnsCertifiedSubnetCatalogVersionPolicy::RequireLatestObserved,
    );

    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &stale_projection, &publication),
        Err(NnsCertifiedSubnetCatalogCacheError::Projection(
            NnsRegistrySubnetCatalogProjectionError::StaleArchiveCertificate { .. }
        ))
    ));
    assert!(!cache_directory.exists());
    assert!(matches!(
        load_nns_certified_subnet_catalog(&archive, &stale_projection, &publish_invalid),
        Err(NnsCertifiedSubnetCatalogCacheError::Projection(
            NnsRegistrySubnetCatalogProjectionError::StaleArchiveCertificate { .. }
        ))
    ));
    assert!(!cache_directory.exists());
    let _ = fs::remove_dir_all(root);
}
