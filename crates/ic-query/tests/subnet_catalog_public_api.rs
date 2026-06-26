use ic_query::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, ClassificationSource, GeographicScope, MAINNET_NETWORK,
    MAINNET_REGISTRY_CANISTER_ID, ResolveAs, ResolvedSubnetSubject, RoutingRange, SubnetCatalog,
    SubnetInfo, SubnetKind, SubnetSpecialization, catalog_to_pretty_json, parse_catalog_json,
};

#[test]
fn public_subnet_catalog_api_parses_and_resolves_without_host() {
    let catalog = SubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 123_456,
        fetched_at: "2026-06-26T00:00:00Z".to_string(),
        fetched_by: "fixture".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        subnets: vec![SubnetInfo {
            subnet_principal: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
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
            start_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            end_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            subnet_principal: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        }],
    };

    let json = catalog_to_pretty_json(&catalog).expect("catalog serializes");
    let parsed = parse_catalog_json(&json).expect("catalog parses");

    let subnet = parsed
        .resolve_principal("rwlgt-iiaaa-aaaaa-aaaaa-cai", Some(ResolveAs::Subnet))
        .expect("subnet resolves");
    assert_eq!(subnet.resolved_as, ResolvedSubnetSubject::Subnet);
    assert_eq!(subnet.subnet.subnet_label, "fiduciary");

    let canister = parsed
        .resolve_principal("ryjl3-tyaaa-aaaaa-aaaba-cai", Some(ResolveAs::Canister))
        .expect("canister resolves through routing range");
    assert_eq!(canister.resolved_as, ResolvedSubnetSubject::Canister);
    assert_eq!(
        canister.matched_canister_principal.as_deref(),
        Some("ryjl3-tyaaa-aaaaa-aaaba-cai")
    );
}
