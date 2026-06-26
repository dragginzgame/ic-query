use ic_query::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, ClassificationSource, GeographicScope, MAINNET_NETWORK,
    MAINNET_REGISTRY_CANISTER_ID, ResolveAs, ResolvedSubnetSubject, RoutingRange, SubnetCatalog,
    SubnetInfo, SubnetKind, SubnetSpecialization, catalog_to_pretty_json, parse_catalog_json,
};
#[cfg(feature = "host")]
use ic_query::subnet_catalog::{
    DEFAULT_REFRESH_LOCK_STALE_SECONDS, DEFAULT_STALE_AFTER_SECONDS,
    DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, SubnetCatalogCacheRequest, SubnetCatalogFilters,
    SubnetCatalogHostError, SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogRefreshReport,
    SubnetCatalogRefreshRequest, SubnetCatalogSubnetRow, build_subnet_catalog_info_report,
    build_subnet_catalog_list_report, load_or_refresh_subnet_catalog, refresh_subnet_catalog,
    subnet_catalog_info_report_text, subnet_catalog_list_report_text,
    subnet_catalog_list_report_verbose_text, subnet_catalog_path, subnet_catalog_refresh_lock_path,
    subnet_catalog_refresh_report_text,
};
#[cfg(feature = "host")]
use std::{
    fs,
    path::{Path, PathBuf},
};

const SUBNET_A: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
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

#[cfg(feature = "host")]
#[test]
fn public_subnet_catalog_host_api_loads_cached_catalog_for_downstream_resolvers() {
    let root = temp_root("subnet-catalog-host-public-api");
    let path = write_fixture_catalog(&root);
    let request = host_cache_request(&root);
    let cached = load_or_refresh_subnet_catalog(
        &request,
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        unix_secs_for_test(),
    )
    .expect("load cached catalog");
    let resolved = cached
        .catalog
        .resolve_principal(CANISTER_A, Some(ResolveAs::Canister))
        .expect("resolve canister");

    let _ = fs::remove_dir_all(root);
    assert_eq!(cached.path, path);
    assert_eq!(resolved.subnet.subnet_principal, SUBNET_A);
    assert_eq!(resolved.subnet.subnet_kind.as_str(), "application");
}

#[cfg(feature = "host")]
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
    assert_eq!(
        refresh_request.lock_stale_after_seconds,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS
    );
    assert!(refresh_api_accepts_public_types(
        refresh_subnet_catalog,
        &refresh_request
    ));
}

#[cfg(feature = "host")]
type SubnetCatalogRefreshFn =
    fn(&SubnetCatalogRefreshRequest) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError>;

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
#[must_use]
fn host_cache_request(root: &Path) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest {
        icp_root: root.to_path_buf(),
        network: MAINNET_NETWORK.to_string(),
    }
}

#[cfg(feature = "host")]
#[must_use]
fn host_info_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogInfoRequest {
    SubnetCatalogInfoRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        input: CANISTER_A.to_string(),
        forced: Some(ResolveAs::Canister),
        now_unix_secs,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
    }
}

#[cfg(feature = "host")]
#[must_use]
fn host_list_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        now_unix_secs,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
        filters: SubnetCatalogFilters::default(),
        show_ranges: true,
        range_limit: 10,
        range_offset: 0,
    }
}

#[cfg(feature = "host")]
#[must_use]
fn host_refresh_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        now_unix_secs,
        lock_stale_after_seconds: DEFAULT_REFRESH_LOCK_STALE_SECONDS,
        dry_run: true,
        output_path: None,
    }
}

#[cfg(feature = "host")]
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
        fetched_at: "2026-06-26T00:00:00Z".to_string(),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        fetched_by: "fixture".to_string(),
        dry_run: true,
        wrote_catalog: false,
        replaced_existing_catalog: true,
        subnet_count: 1,
        routing_range_count: 1,
    }
}

#[cfg(feature = "host")]
#[must_use]
fn refresh_api_accepts_public_types(
    _refresh: SubnetCatalogRefreshFn,
    request: &SubnetCatalogRefreshRequest,
) -> bool {
    request.dry_run
}

#[cfg(feature = "host")]
#[must_use]
fn temp_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ic-query-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[cfg(feature = "host")]
#[must_use]
fn unix_secs_for_test() -> u64 {
    std::time::SystemTime::UNIX_EPOCH
        .elapsed()
        .expect("system time after unix epoch")
        .as_secs()
}

#[must_use]
fn fixture_catalog() -> SubnetCatalog {
    SubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 123_456,
        fetched_at: "2026-06-26T00:00:00Z".to_string(),
        fetched_by: "fixture".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        subnets: vec![SubnetInfo {
            subnet_principal: SUBNET_A.to_string(),
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
    }
}
