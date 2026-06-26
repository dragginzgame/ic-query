use ic_query::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, ClassificationSource, GeographicScope, MAINNET_NETWORK,
    MAINNET_REGISTRY_CANISTER_ID, ResolveAs, ResolvedSubnetSubject, RoutingRange, SubnetCatalog,
    SubnetInfo, SubnetKind, SubnetSpecialization, catalog_to_pretty_json, parse_catalog_json,
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
    use ic_query::subnet_catalog::{
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, SubnetCatalogCacheRequest,
        load_or_refresh_subnet_catalog, subnet_catalog_path,
    };
    use std::fs;

    let root = temp_root("subnet-catalog-host-public-api");
    let path = subnet_catalog_path(&root, MAINNET_NETWORK);
    fs::create_dir_all(path.parent().expect("catalog parent")).expect("create catalog parent");
    fs::write(
        &path,
        catalog_to_pretty_json(&fixture_catalog()).expect("catalog serializes"),
    )
    .expect("write catalog");

    let request = SubnetCatalogCacheRequest {
        icp_root: root.clone(),
        network: MAINNET_NETWORK.to_string(),
    };
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
#[must_use]
fn temp_root(name: &str) -> std::path::PathBuf {
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
