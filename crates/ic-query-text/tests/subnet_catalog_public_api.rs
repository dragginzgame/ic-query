#[cfg(feature = "subnet-catalog-host")]
mod support;

#[cfg(feature = "subnet-catalog-host")]
use ic_query::report::CollectionSourceRequest;
use ic_query::subnet_catalog::CANISTER_RANGES_KEY_PREFIX;
use ic_query::subnet_catalog::CATALOG_SCHEMA_VERSION;
use ic_query::subnet_catalog::CLASSIFICATION_SCHEMA_VERSION;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::CacheDisposition;
use ic_query::subnet_catalog::CatalogAssurance;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::CatalogSnapshotAuthorityEvidence;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::CatalogSourceSelection;
use ic_query::subnet_catalog::ClassificationSource;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::DEFAULT_STALE_AFTER_SECONDS;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT;
use ic_query::subnet_catalog::GeographicScope;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::LiveSubnetCatalogSource;
use ic_query::subnet_catalog::MAINNET_NETWORK;
use ic_query::subnet_catalog::MAINNET_REGISTRY_CANISTER_ID;
use ic_query::subnet_catalog::RESOLVER_SCHEMA_VERSION;
use ic_query::subnet_catalog::ROUTING_TABLE_KEY;
use ic_query::subnet_catalog::RawSubnetCatalog;
use ic_query::subnet_catalog::ResolveAs;
use ic_query::subnet_catalog::ResolvedSubnetSubject;
use ic_query::subnet_catalog::RoutingRange;
use ic_query::subnet_catalog::SUBNET_LIST_KEY;
use ic_query::subnet_catalog::SUBNET_RECORD_KEY_PREFIX;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogCacheRequest;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogDetailedSourceFuture;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogFailureCacheDisposition;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogFilters;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogHostError;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogInfoReport;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogInfoRequest;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogListReport;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogListRequest;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogLoadFailure;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogLoadRequest;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogLoadStage;
use ic_query::subnet_catalog::SubnetCatalogProvenance;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogRefreshReport;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogRefreshRequest;
use ic_query::subnet_catalog::SubnetCatalogRegistryRecordEvidence;
use ic_query::subnet_catalog::SubnetCatalogRegistryRecordSubject;
use ic_query::subnet_catalog::SubnetCatalogRegistryValueEncoding;
use ic_query::subnet_catalog::SubnetCatalogRoutingSource;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogSource;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogSourceFailure;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogSourceFuture;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogSubject;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::SubnetCatalogSubnetRow;
use ic_query::subnet_catalog::SubnetInfo;
use ic_query::subnet_catalog::SubnetKind;
use ic_query::subnet_catalog::SubnetSpecialization;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::build_subnet_catalog_info_report;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::build_subnet_catalog_list_report;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::build_subnet_catalog_list_report_with_source_and_progress;
use ic_query::subnet_catalog::catalog_to_pretty_json;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::fetch_subnet_catalog_async;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::load_cached_subnet_catalog;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::load_subnet_catalog;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::load_subnet_catalog_with_source;
use ic_query::subnet_catalog::parse_catalog_json;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::refresh_subnet_catalog_with_source;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::subnet_catalog_path;
#[cfg(feature = "subnet-catalog-host")]
use ic_query::subnet_catalog::subnet_catalog_refresh_lock_path;
#[cfg(feature = "subnet-catalog-host")]
use ic_query_text::subnet_catalog::subnet_catalog_info_report_text;
#[cfg(feature = "subnet-catalog-host")]
use ic_query_text::subnet_catalog::subnet_catalog_list_report_text;
#[cfg(feature = "subnet-catalog-host")]
#[cfg(feature = "subnet-catalog-host")]
use ic_query_text::subnet_catalog::subnet_catalog_refresh_report_text;
#[cfg(all(feature = "subnet-catalog-host", unix))]
use std::os::unix::fs::PermissionsExt;
#[cfg(feature = "subnet-catalog-host")]
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
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
    assert_eq!(subnet.subnet.classification.subnet_label, "fiduciary");

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
#[tokio::test(flavor = "current_thread")]
async fn public_subnet_catalog_host_api_loads_cached_catalog_for_downstream_resolvers() {
    let root = temp_root("subnet-catalog-host-public-api");
    let path = write_fixture_catalog(&root);
    let request =
        SubnetCatalogLoadRequest::cache_only(host_cache_request(&root), unix_secs_for_test());
    let cached = load_cached_subnet_catalog(&request).expect("load cached catalog");
    let events = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&events);
    let source = LiveSubnetCatalogSource::with_progress(
        move |_: ic_query::subnet_catalog::SubnetCatalogProgress| {
            observed.fetch_add(1, Ordering::Relaxed);
        },
    )
    .with_history_cache(root.join("unused-history-root"));
    let reused = load_subnet_catalog_with_source(&request, &source)
        .await
        .expect("reusable source cache hit");
    assert_eq!(events.load(Ordering::Relaxed), 0);
    assert!(!root.join("unused-history-root").exists());
    assert_eq!(cached.snapshot_authority(), reused.snapshot_authority());

    let resolved = cached
        .catalog
        .resolve_canister_route(CANISTER_A)
        .expect("resolve canister");

    let _ = fs::remove_dir_all(root);
    assert_eq!(cached.path, path);
    assert_eq!(resolved.subnet.to_text(), SUBNET_A);
    assert_eq!(resolved.subnet_info.subnet_principal, SUBNET_A);
    assert_eq!(
        resolved.subnet_info.classification.subnet_kind,
        SubnetKind::Application
    );
    assert_eq!(resolved.registry_version, 123_456);
    assert_eq!(cached.disposition, CacheDisposition::CacheHit);
    let authority: CatalogSnapshotAuthorityEvidence = cached.snapshot_authority();
    assert_eq!(authority.registry_version, 123_456);
    assert_eq!(authority.assurance, CatalogAssurance::UncertifiedQuery);
    assert_eq!(authority, cached.catalog.snapshot_authority());
}

#[cfg(feature = "subnet-catalog-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_subnet_catalog_host_api_builds_reports_and_renders_text() {
    let root = temp_root("subnet-catalog-host-report-public-api");
    let path = write_fixture_catalog(&root);
    let cache = host_cache_request(&root);
    let now_unix_secs = unix_secs_for_test();
    let info_request = host_info_request(&cache, now_unix_secs);
    let info_report: SubnetCatalogInfoReport = build_subnet_catalog_info_report(&info_request)
        .await
        .expect("build info report");
    let info_text = subnet_catalog_info_report_text(&info_report);

    let list_request = host_list_request(&cache, now_unix_secs);
    let list_report: SubnetCatalogListReport = build_subnet_catalog_list_report(&list_request)
        .await
        .expect("build list report");
    let row: &SubnetCatalogSubnetRow = list_report.subnets.first().expect("subnet row");
    let list_text =
        subnet_catalog_list_report_text(&list_report, ic_query_text::TextOptions::default());
    let list_verbose_text =
        subnet_catalog_list_report_text(&list_report, ic_query_text::TextOptions { verbose: true });

    let refresh_request = host_refresh_request(&cache, now_unix_secs);
    let refresh_report = fixture_refresh_report(&root, &path);
    let refresh_text = subnet_catalog_refresh_report_text(&refresh_report);

    let _ = fs::remove_dir_all(root);
    assert_eq!(info_report.subnet_principal, SUBNET_A);
    assert_eq!(info_report.metadata.registry_query_call_count, 5);
    assert!(info_report.metadata.agreement_digest.is_none());
    assert!(info_text.contains("resolved_as: canister"));
    assert_eq!(row.subnet_principal, SUBNET_A);
    assert_eq!(list_report.metadata.registry_query_call_count, 5);
    assert_eq!(row.ranges_shown, 1);
    assert!(list_text.contains("catalog: ic version 123456 stale no"));
    assert!(list_verbose_text.contains(CANISTER_A));
    assert!(refresh_text.contains("dry_run: yes"));
    assert!(refresh_text.contains("assurance: uncertified_query"));
    assert!(refresh_text.contains("source_endpoints: https://icp-api.io"));
    assert!(refresh_text.contains("agreement_digest: -"));
    assert!(refresh_text.contains("registry_query_call_count: 5"));

    let async_refresh = futures::executor::block_on(refresh_subnet_catalog_with_source(
        &refresh_request,
        &FixtureSubnetCatalogSource,
    ))
    .expect("public async refresh");
    assert_eq!(async_refresh.metadata.registry_query_call_count, 5);
}

#[cfg(feature = "subnet-catalog-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_subnet_catalog_host_api_accepts_custom_source_adapter() {
    let root = temp_root("subnet-catalog-host-custom-source-public-api");
    let cache = host_cache_request(&root);
    let now_unix_secs = unix_secs_for_test();
    let request = host_list_request(&cache, now_unix_secs);

    let mut load_request = SubnetCatalogLoadRequest::cache_only(cache, now_unix_secs);
    load_request.policy = request.read_policy.clone();

    let loaded = futures::executor::block_on(load_subnet_catalog_with_source(
        &load_request,
        &FixtureSubnetCatalogSource,
    ))
    .expect("public async load");

    let report = build_subnet_catalog_list_report_with_source_and_progress(
        &request,
        &FixtureSubnetCatalogSource,
        &mut |_| {},
    )
    .await
    .expect("build list report from custom source");

    let _ = fs::remove_dir_all(root);
    assert_eq!(loaded.disposition, CacheDisposition::RefreshedMissing);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.subnets.len(), 1);
    assert_eq!(report.subnets[0].subnet_principal, SUBNET_A);
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn public_detailed_load_api_exposes_typed_failure_provenance() {
    fn accepts_async_loaders(
        _load: impl AsyncFn(
            &SubnetCatalogLoadRequest,
        ) -> Result<
            ic_query::subnet_catalog::CatalogLoadOutcome,
            Box<SubnetCatalogLoadFailure>,
        >,
        _source: impl AsyncFn(
            &SubnetCatalogLoadRequest,
            &dyn SubnetCatalogSource,
        ) -> Result<
            ic_query::subnet_catalog::CatalogLoadOutcome,
            Box<SubnetCatalogLoadFailure>,
        >,
    ) {
    }

    let root = temp_root("subnet-catalog-host-detailed-public-api");
    let request =
        SubnetCatalogLoadRequest::cache_only(host_cache_request(&root), unix_secs_for_test());

    let failure = load_cached_subnet_catalog(&request).expect_err("cache missing");

    assert_eq!(failure.stage, SubnetCatalogLoadStage::CacheAbsence);
    assert_eq!(
        failure.cache_disposition,
        SubnetCatalogFailureCacheDisposition::CacheMissing
    );
    assert_eq!(failure.evidence.registry_version, None);
    assert_eq!(failure.evidence.returned_registry_value_version, None);
    assert_eq!(failure.evidence.source_endpoint, None);
    assert_eq!(failure.evidence.assurance, None);
    assert_eq!(failure.evidence.registry_records, Vec::new());
    assert_eq!(failure.request.network, MAINNET_NETWORK);
    assert!(matches!(
        failure.source,
        SubnetCatalogHostError::MissingCatalog { .. }
    ));

    accepts_async_loaders(load_subnet_catalog, load_subnet_catalog_with_source);
    let _ = load_subnet_catalog;
    let _ = load_subnet_catalog_with_source;
    let _ = fs::remove_dir_all(root);
}

#[test]
fn public_registry_value_encoding_exposes_stable_labels() {
    assert_eq!(
        SubnetCatalogRegistryValueEncoding::Inline.as_str(),
        "inline"
    );
    assert_eq!(
        SubnetCatalogRegistryValueEncoding::Chunked.as_str(),
        "chunked"
    );
}

#[test]
fn public_registry_evidence_builders_derive_exact_subject_keys() {
    let subnet = ic_platform::candid::contract::Principal::from_text(SUBNET_A)
        .expect("fixture Subnet principal");
    let range_start =
        ic_platform::candid::contract::Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 1, 1, 1]);

    let subnet_list = SubnetCatalogRegistryRecordSubject::subnet_list();
    let legacy_routing = SubnetCatalogRegistryRecordSubject::legacy_routing_table();
    let subnet_record = SubnetCatalogRegistryRecordSubject::subnet_record(subnet);
    let routing_shard = SubnetCatalogRegistryRecordSubject::canister_ranges(range_start);

    assert_eq!(subnet_list.key, SUBNET_LIST_KEY);
    assert_eq!(legacy_routing.key, ROUTING_TABLE_KEY);
    assert_eq!(
        subnet_record.key,
        format!("{SUBNET_RECORD_KEY_PREFIX}{SUBNET_A}")
    );
    assert!(routing_shard.key.starts_with(CANISTER_RANGES_KEY_PREFIX));
    assert_eq!(routing_shard.canister_range_start, Some(range_start));

    let evidence = SubnetCatalogRegistryRecordEvidence::uncertified_query(
        routing_shard,
        63_438,
        63_420,
        1_780_531_200_000_000_000,
        "https://icp-api.io",
        SubnetCatalogRegistryValueEncoding::Chunked,
    );
    assert_eq!(evidence.assurance, CatalogAssurance::UncertifiedQuery);
    assert_eq!(evidence.requested_registry_version, 63_438);
    assert_eq!(evidence.returned_registry_version, 63_420);
}

#[cfg(feature = "subnet-catalog-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_custom_source_can_supply_exact_failure_version_and_subject() {
    struct DetailedSource;

    impl SubnetCatalogSource for DetailedSource {
        fn fetch_catalog<'a>(
            &'a self,
            _request: &'a CollectionSourceRequest,
        ) -> SubnetCatalogSourceFuture<'a> {
            Box::pin(async {
                Err(SubnetCatalogHostError::Catalog(
                    ic_query::subnet_catalog::CatalogError::EmptySubnets,
                ))
            })
        }

        fn fetch_catalog_detailed<'a>(
            &'a self,
            _request: &'a CollectionSourceRequest,
        ) -> SubnetCatalogDetailedSourceFuture<'a> {
            Box::pin(async {
                Err(SubnetCatalogSourceFailure::new(
                    Some(700_008),
                    Some(SubnetCatalogSubject::RegistryRecord(
                        SubnetCatalogRegistryRecordSubject::legacy_routing_table(),
                    )),
                    SubnetCatalogHostError::Catalog(
                        ic_query::subnet_catalog::CatalogError::EmptyRoutingRanges,
                    ),
                ))
            })
        }
    }

    let root = temp_root("subnet-catalog-host-detailed-source-public-api");
    let mut request =
        SubnetCatalogLoadRequest::cache_only(host_cache_request(&root), unix_secs_for_test());
    request.policy = ic_query::subnet_catalog::CatalogReadPolicy::ForceRefresh {
        source: CatalogSourceSelection::uncertified_query(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT),
    };

    let failure = load_subnet_catalog_with_source(&request, &DetailedSource)
        .await
        .expect_err("fixture source fails");

    assert_eq!(failure.evidence.registry_version, Some(700_008));
    assert_eq!(
        failure.evidence.subject,
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject::legacy_routing_table()
        ))
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn public_async_catalog_fetch_rejects_non_mainnet_without_live_io() {
    let request = CollectionSourceRequest::new(
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
    fn fetch_catalog<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> SubnetCatalogSourceFuture<'a> {
        Box::pin(async move {
            assert_eq!(request.endpoint, DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT);
            assert_eq!(request.fetched_by, "ic-query");
            assert_ne!(request.fetched_at, "");

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
        })
    }
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn write_fixture_catalog(root: &Path) -> PathBuf {
    let path = subnet_catalog_path(root, MAINNET_NETWORK);
    create_managed_fixture_parent(root, &path);
    fs::write(
        &path,
        catalog_to_pretty_json(&fixture_catalog()).expect("catalog serializes"),
    )
    .expect("write catalog");
    #[cfg(unix)]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("secure catalog file");
    path
}

#[cfg(feature = "subnet-catalog-host")]
fn create_managed_fixture_parent(root: &Path, path: &Path) {
    let parent = path.parent().expect("catalog parent");
    fs::create_dir_all(parent).expect("create catalog parent");
    #[cfg(unix)]
    {
        let mut directory = root.to_path_buf();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .expect("secure catalog root");
        for component in parent.strip_prefix(root).expect("parent beneath root") {
            directory.push(component);
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
                .expect("secure catalog directory");
        }
    }
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
    let mut configured_request = SubnetCatalogInfoRequest::new(
        cache.clone(),
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        CANISTER_A,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    configured_request.forced = Some(ResolveAs::Canister);
    configured_request
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn host_list_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogListRequest {
    let mut configured_request = SubnetCatalogListRequest::new(
        cache.clone(),
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    configured_request.filters = SubnetCatalogFilters::default();
    configured_request.show_ranges = true;
    configured_request.range_limit = 10;
    configured_request
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn host_refresh_request(
    cache: &SubnetCatalogCacheRequest,
    now_unix_secs: u64,
) -> SubnetCatalogRefreshRequest {
    let mut configured_request = SubnetCatalogRefreshRequest::new(
        cache.clone(),
        CatalogSourceSelection::uncertified_query(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT),
        now_unix_secs,
    );
    configured_request.dry_run = true;
    configured_request
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
        metadata: ic_query::subnet_catalog::SubnetCatalogReportMetadata {
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 123_456,
            assurance: CatalogAssurance::UncertifiedQuery,
            source_endpoints: vec![DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string()],
            agreement_digest: None,
            registry_query_call_count: 5,
            routing_source: SubnetCatalogRoutingSource::LegacyRoutingTable,
            registry_records: Vec::new(),
            catalog_digest: "00".repeat(32),
            fetched_at: "2026-06-26T00:00:00Z".to_string(),
            collector_version: "test".to_string(),
            classification_schema_version: 1,
            classification_policy_digest: "00".repeat(32),
            resolver_schema_version: 1,
            resolver_backend: "local-nns-subnet-catalog".to_string(),
        },
        fetched_by: "fixture".to_string(),
        dry_run: true,
        wrote_catalog: false,
        replaced_existing_catalog: true,
        subnet_count: 1,
        routing_range_count: 1,
    }
}

#[cfg(feature = "subnet-catalog-host")]
#[must_use]
fn temp_root(name: &str) -> PathBuf {
    let mut path = crate::support::scratch_root();
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
            agreement_digest: None,
            registry_query_call_count: 5,
            routing_source: SubnetCatalogRoutingSource::LegacyRoutingTable,
            registry_records: Vec::new(),
            fetched_at: "2026-06-26T00:00:00Z".to_string(),
            fetched_by: "fixture".to_string(),
            collector_version: "test".to_string(),
            classification_schema_version: CLASSIFICATION_SCHEMA_VERSION,
            classification_policy_digest: "00".repeat(32),
            resolver_schema_version: RESOLVER_SCHEMA_VERSION,
            resolver_backend: "local-nns-subnet-catalog".to_string(),
        },
        catalog_digest: "00".repeat(32),
        subnets: vec![SubnetInfo {
            classification: ic_query::subnet_catalog::SubnetClassification {
                registry_subnet_type: 1,
                subnet_kind: SubnetKind::Application,
                subnet_kind_source: ClassificationSource::Registry,
                subnet_specialization: SubnetSpecialization::Fiduciary,
                subnet_specialization_source: ClassificationSource::Curated,
                geographic_scope: GeographicScope::Global,
                geographic_scope_source: ClassificationSource::Curated,
                subnet_label: "fiduciary".to_string(),
                subnet_label_source: ClassificationSource::Curated,
            },
            subnet_principal: SUBNET_A.to_string(),
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
    {
        let endpoint = catalog.provenance.source_endpoints[0].clone();
        let registry_version = catalog.provenance.registry_version;
        let evidence = |record| {
            SubnetCatalogRegistryRecordEvidence::uncertified_query(
                record,
                registry_version,
                registry_version - 1,
                1_780_531_200_000_000_000,
                &endpoint,
                SubnetCatalogRegistryValueEncoding::Inline,
            )
        };
        let subnet = ic_platform::candid::contract::Principal::from_text(SUBNET_A)
            .expect("fixture Subnet principal");
        catalog.provenance.registry_records = vec![
            evidence(SubnetCatalogRegistryRecordSubject::subnet_list()),
            evidence(SubnetCatalogRegistryRecordSubject::legacy_routing_table()),
            evidence(SubnetCatalogRegistryRecordSubject::subnet_record(subnet)),
        ];
    }
    #[cfg(feature = "subnet-catalog-host")]
    catalog
        .canonicalize_and_seal()
        .expect("fixture catalog seals");
    catalog
}
