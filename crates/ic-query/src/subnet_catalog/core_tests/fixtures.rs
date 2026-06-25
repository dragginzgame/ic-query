use crate::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, ClassificationSource, GeographicScope, MAINNET_NETWORK,
    MAINNET_REGISTRY_CANISTER_ID, RoutingRange, SubnetCatalog, SubnetInfo, SubnetKind,
    SubnetSpecialization, principal_bytes,
};

pub(super) const SUBNET_A: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
pub(super) const SUBNET_B: &str = "aaaaa-aa";

pub(super) fn fixture_catalog() -> SubnetCatalog {
    SubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 123_456,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "fixture".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        subnets: vec![
            SubnetInfo {
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
            },
            SubnetInfo {
                subnet_principal: SUBNET_B.to_string(),
                subnet_kind: SubnetKind::Application,
                subnet_kind_source: ClassificationSource::Registry,
                subnet_specialization: SubnetSpecialization::European,
                subnet_specialization_source: ClassificationSource::Curated,
                geographic_scope: GeographicScope::Europe,
                geographic_scope_source: ClassificationSource::Curated,
                subnet_label: "european".to_string(),
                subnet_label_source: ClassificationSource::Curated,
                node_count: Some(13),
                charges_apply_by_default: true,
            },
        ],
        routing_ranges: vec![RoutingRange {
            start_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            end_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            subnet_principal: SUBNET_A.to_string(),
        }],
    }
}

pub(super) fn sorted_principals<const N: usize>(ids: [&str; N]) -> Vec<String> {
    let mut ids = ids.map(str::to_string).to_vec();
    ids.sort_by(|left, right| {
        principal_bytes(left, "test")
            .expect("valid left")
            .cmp(&principal_bytes(right, "test").expect("valid right"))
    });
    ids
}
