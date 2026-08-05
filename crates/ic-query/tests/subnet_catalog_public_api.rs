#[cfg(feature = "subnet-catalog-host")]
use ic_query::nns::NnsSourceRequest;
use ic_query::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, CLASSIFICATION_SCHEMA_VERSION, CatalogAssurance, ClassificationSource,
    GeographicScope, MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, RESOLVER_SCHEMA_VERSION,
    RawSubnetCatalog, ResolveAs, ResolvedSubnetSubject, RoutingRange, SubnetCatalogProvenance,
    SubnetInfo, SubnetKind, SubnetSpecialization, catalog_to_pretty_json, parse_catalog_json,
};
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::{
    CacheDisposition, DEFAULT_REFRESH_LOCK_STALE_SECONDS, DEFAULT_STALE_AFTER_SECONDS,
    DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, SubnetCatalogCacheRequest, SubnetCatalogFilters,
    SubnetCatalogHostError, SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogLoadRequest,
    SubnetCatalogRefreshReport, SubnetCatalogRefreshRequest, SubnetCatalogSource,
    SubnetCatalogSubnetRow, build_subnet_catalog_info_report, build_subnet_catalog_list_report,
    build_subnet_catalog_list_report_with_source, fetch_subnet_catalog_async,
    load_cached_subnet_catalog, refresh_subnet_catalog, subnet_catalog_info_report_text,
    subnet_catalog_list_report_text, subnet_catalog_list_report_verbose_text, subnet_catalog_path,
    subnet_catalog_refresh_lock_path, subnet_catalog_refresh_report_text,
};
#[cfg(feature = "subnet-catalog-host")]
use std::{
    fs,
    path::{Path, PathBuf},
};

const SUBNET_A: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
const CANISTER_A: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[test]
fn public_subnet_catalog_api_parses_and_resolves_without_host() {
    let catalog = fixture_catalog();
    let json = catalog_to_pretty_json(&catalog).expect("catalog serializes");
    let parsed = parse_catalog_json(&json).expect("catalog parses");

    let subnet = parsed
        .resolve_principal(SUBNET_A, Some(ResolveAs::Subnet))
        .expect("subnet resolves");
    assert_eq!(subnet.resolved_as, ResolvedSubnetSubject::Subnet);
    assert_eq!(subnet.subnet.subnet_label, "fiduciary");

    let canister = parsed
        .resolve_principal(CANISTER_A, Some(ResolveAs::Canister))
        .expect("canister resolves through routing range");
    assert_eq!(canister.resolved_as, ResolvedSubnetSubject::Canister);
    assert_eq!(
        canister.matched_canister_principal.as_deref(),
        Some(CANISTER_A)
    );
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn public_subnet_catalog_host_api_loads_cached_catalog_for_downstream_resolvers() {
    let root = temp_root("subnet-catalog-host-public-api");
    let path = write_fixture_catalog(&root);
    let request =
        SubnetCatalogLoadRequest::cache_only(host_cache_request(&root), unix_secs_for_test());
    let cached = load_cached_subnet_catalog(&request).expect("load cached catalog");
    let resolved = cached
        .catalog
        .resolve_canister_route(CANISTER_A)
        .expect("resolve canister");

    let _ = fs::remove_dir_all(root);
    assert_eq!(cached.path, path);
    assert_eq!(resolved.subnet.to_text(), SUBNET_A);
    assert_eq!(resolved.registry_version, 123_456);
    assert_eq!(cached.disposition, CacheDisposition::CacheHit);
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn public_subnet_catalog_host_api_builds_reports_and_renders_text() {
    let root = temp_root("subnet-catalog-host-report-public-api");
    let path = write_fixture_catalog(&root);
    let cache = host_cache_request(&root);
    let now_unix_secs = unix_secs_for_test();
    let info_request = host_info_request(&cache, now_unix_secs);
    let info_report: SubnetCatalogInfoReport =
        build_subnet_catalog_info_report(&info_request).expect("build info report");
    let info_text = subnet_catalog_info_report_text(&info_report);

    let list_request = host_list_request(&cache, now_unix_secs);
    let list_report: SubnetCatalogListReport =
        build_subnet_catalog_list_report(&list_request).expect("build list report");
    let row: &SubnetCatalogSubnetRow = list_report.subnets.first().expect("subnet row");
    let list_text = subnet_catalog_list_report_text(&list_report);
    let list_verbose_text = subnet_catalog_list_report_verbose_text(&list_report);

    let refresh_request = host_refresh_request(&cache, now_unix_secs);
    let refresh_report = fixture_refresh_report(&root, &path);
    let refresh_text = subnet_catalog_refresh_report_text(&refresh_report);

    let _ = fs::remove_dir_all(root);
    assert_eq!(info_report.subnet_principal, SUBNET_A);
    assert!(info_text.contains("resolved_as: canister"));
    assert_eq!(row.subnet_principal, SUBNET_A);
    assert_eq!(row.ranges_shown, 1);
    assert!(list_text.contains("catalog: ic version 123456 stale no"));
    assert!(list_verbose_text.contains(CANISTER_A));
    assert!(refresh_text.contains("dry_run: yes"));
    assert!(refresh_text.contains("assurance: uncertified_query"));
    assert!(refresh_text.contains("source_endpoints: https://icp-api.io"));
    assert_eq!(
        refresh_request.lock_stale_after_seconds,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(refresh_api_accepts_public_types(
        refresh_subnet_catalog,
        &refresh_request
    ));
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn public_subnet_catalog_host_api_accepts_custom_source_adapter() {
    let root = temp_root("subnet-catalog-host-custom-source-public-api");
    let cache = host_cache_request(&root);
    let now_unix_secs = unix_secs_for_test();
    let request = host_list_request(&cache, now_unix_secs);

    let report =
        build_subnet_catalog_list_report_with_source(&request, &FixtureSubnetCatalogSource)
            .expect("build list report from custom source");

    let _ = fs::remove_dir_all(root);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.subnets.len(), 1);
    assert_eq!(report.subnets[0].subnet_principal, SUBNET_A);
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn public_async_catalog_fetch_rejects_non_mainnet_without_live_io() {
    let request = NnsSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-06-26T00:00:00Z",
        "fixture",
    );

    let error = futures::executor::block_on(fetch_subnet_catalog_async(&request))
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        SubnetCatalogHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[cfg(feature = "subnet-catalog-host")]
struct FixtureSubnetCatalogSource;

#[cfg(feature = "subnet-catalog-host")]
impl SubnetCatalogSource for FixtureSubnetCatalogSource {
    fn fetch_catalog(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<RawSubnetCatalog, SubnetCatalogHostError> {
        assert_eq!(request.endpoint, DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT);
        assert_eq!(request.fetched_by, "ic-query");
        assert!(!request.fetched_at.is_empty());

        let mut catalog = fixture_catalog();
        catalog.provenance.source_endpoints = vec![request.endpoint.clone()];
        catalog
            .provenance
            .fetched_at
            .clone_from(&request.fetched_at);
        catalog
            .provenance
            .fetched_by
            .clone_from(&request.fetched_by);
        catalog
            .canonicalize_and_seal()
            .expect("fixture catalog reseals");
        Ok(catalog)
    }
}

#[cfg(feature = "subnet-catalog-host")]
type SubnetCatalogRefreshFn =
    fn(&SubnetCatalogRefreshRequest) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError>;

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn write_fixture_catalog(root: &Path) -> PathBuf {
    let path = subnet_catalog_path(root, MAINNET_NETWORK);
    fs::create_dir_all(path.parent().expect("catalog parent")).expect("create catalog parent");
    fs::write(
        &path,
        catalog_to_pretty_json(&fixture_catalog()).expect("catalog serializes"),
    )
    .expect("write catalog");
    path
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn host_cache_request(root: &Path) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest::new(root, MAINNET_NETWORK)
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn host_info_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogInfoRequest {
    SubnetCatalogInfoRequest::new(
        cache.clone(),
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        CANISTER_A,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_forced(ResolveAs::Canister)
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn host_list_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest::new(
        cache.clone(),
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_filters(SubnetCatalogFilters::default())
    .with_show_ranges(true)
    .with_range_limit(10)
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn host_refresh_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest::new(
        cache.clone(),
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        now_unix_secs,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true)
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn fixture_refresh_report(root: &Path, catalog_path: &Path) -> SubnetCatalogRefreshReport {
    SubnetCatalogRefreshReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        catalog_path: catalog_path.display().to_string(),
        refresh_lock_path: subnet_catalog_refresh_lock_path(root, MAINNET_NETWORK)
            .display()
            .to_string(),
        output_path: None,
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 123_456,
        assurance: CatalogAssurance::UncertifiedQuery,
        source_endpoints: vec![DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string()],
        catalog_digest: "00".repeat(32),
        fetched_at: "2026-06-26T00:00:00Z".to_string(),
        fetched_by: "fixture".to_string(),
        collector_version: "test".to_string(),
        classification_schema_version: 1,
        classification_policy_digest: "00".repeat(32),
        resolver_schema_version: 1,
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        dry_run: true,
        wrote_catalog: false,
        replaced_existing_catalog: true,
        subnet_count: 1,
        routing_range_count: 1,
    }
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn refresh_api_accepts_public_types(
    _refresh: SubnetCatalogRefreshFn,
    request: &SubnetCatalogRefreshRequest,
) -> bool {
    request.dry_run
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn temp_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ic-query-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
const fn unix_secs_for_test() -> u64 {
    1_782_432_100
}

#[must_use]
fn fixture_catalog() -> RawSubnetCatalog {
    let catalog = RawSubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        provenance: SubnetCatalogProvenance {
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 123_456,
            assurance: CatalogAssurance::UncertifiedQuery,
            source_endpoints: vec!["https://icp-api.io".to_string()],
            fetched_at: "2026-06-26T00:00:00Z".to_string(),
            certificate_time: None,
            root_key_digest: None,
            fetched_by: "fixture".to_string(),
            collector_version: "test".to_string(),
            classification_schema_version: CLASSIFICATION_SCHEMA_VERSION,
            classification_policy_digest: "00".repeat(32),
            resolver_schema_version: RESOLVER_SCHEMA_VERSION,
            resolver_backend: "local-nns-subnet-catalog".to_string(),
        },
        catalog_digest: "00".repeat(32),
        subnets: vec![SubnetInfo {
            subnet_principal: SUBNET_A.to_string(),
            registry_subnet_type: 1,
            subnet_kind: SubnetKind::Application,
            subnet_kind_source: ClassificationSource::Registry,
            subnet_specialization: SubnetSpecialization::Fiduciary,
            subnet_specialization_source: ClassificationSource::Curated,
            geographic_scope: GeographicScope::Global,
            geographic_scope_source: ClassificationSource::Curated,
            subnet_label: "fiduciary".to_string(),
            subnet_label_source: ClassificationSource::Curated,
            node_count: Some(34),
            charges_apply_by_default: true,
        }],
        routing_ranges: vec![RoutingRange {
            start_canister_id: CANISTER_A.to_string(),
            end_canister_id: CANISTER_A.to_string(),
            subnet_principal: SUBNET_A.to_string(),
        }],
    };
    #[cfg(feature = "subnet-catalog-host")]
    let mut catalog = catalog;
    #[cfg(feature = "subnet-catalog-host")]
    catalog
        .canonicalize_and_seal()
        .expect("fixture catalog seals");
    catalog
}
