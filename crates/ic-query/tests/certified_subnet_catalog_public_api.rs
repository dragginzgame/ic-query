#[cfg(feature = "certified-subnet-catalog-host")]
use ic_query::nns::registry::{
    NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION,
    NNS_CERTIFIED_SUBNET_CATALOG_CACHE_SCHEMA_VERSION, NnsCertifiedRegistryArchiveLimits,
    NnsCertifiedRegistryArchiveStorageLimits, NnsCertifiedSubnetCatalogCacheLocation,
    NnsCertifiedSubnetCatalogLoadRequest, NnsCertifiedSubnetCatalogReadPolicy,
    NnsRegistryReplayLimits, NnsRegistryReplaySessionLimits,
    nns_certified_registry_archive_manifest_path, nns_certified_subnet_catalog_cache_path,
};
#[cfg(feature = "certified-subnet-catalog-host")]
use std::path::Path;

#[cfg(feature = "certified-subnet-catalog-host")]
#[test]
fn focused_certified_subnet_catalog_surface_is_constructible() {
    let replay_limits = NnsRegistryReplaySessionLimits::new(
        100,
        10,
        100,
        1_000_000,
        NnsRegistryReplayLimits::new(1_000, 1_000_000),
    );
    let storage_limits = NnsCertifiedRegistryArchiveStorageLimits::new(
        100_000,
        NnsCertifiedRegistryArchiveLimits::new(10, 1_000_000, 10_000_000),
    );
    let location = NnsCertifiedSubnetCatalogCacheLocation::new(
        "/tmp/ic-query-focused-certified-catalog",
        "/tmp/ic-query-focused-certified-catalog/catalog",
        1_000_000,
    );
    let request = NnsCertifiedSubnetCatalogLoadRequest::publish_missing(location.clone(), 300);

    assert_eq!(NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION, 1);
    assert_eq!(NNS_CERTIFIED_SUBNET_CATALOG_CACHE_SCHEMA_VERSION, 1);
    assert_eq!(replay_limits.max_registry_versions, 100);
    assert_eq!(storage_limits.max_manifest_bytes, 100_000);
    assert_eq!(request.location, location);
    assert_eq!(
        request.policy,
        NnsCertifiedSubnetCatalogReadPolicy::PublishMissing {
            lock_stale_after_seconds: 300,
        }
    );
    assert_eq!(
        nns_certified_registry_archive_manifest_path(Path::new("/archive")),
        Path::new("/archive/manifest.json")
    );
    assert_eq!(
        nns_certified_subnet_catalog_cache_path(Path::new("/catalog")),
        Path::new("/catalog/catalog.json")
    );
}
