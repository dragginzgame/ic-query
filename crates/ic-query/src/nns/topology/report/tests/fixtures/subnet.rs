use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID,
    SubnetCatalogListReport, SubnetCatalogRefreshReport, SubnetCatalogSubnetRow, SubnetKind,
    SubnetSpecialization,
};

pub(in crate::nns::topology::report::tests) fn subnet_report_fixture() -> SubnetCatalogListReport {
    SubnetCatalogListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        catalog_path: "catalog.json".to_string(),
        catalog_schema_version: 1,
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        catalog_stale: false,
        stale_reason: "fresh".to_string(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        subnets: vec![
            subnet_row("pzp6e", SubnetKind::Application, 2, 2),
            subnet_row("tdb26", SubnetKind::System, 1, 1),
        ],
    }
}

pub(in crate::nns::topology::report::tests) fn subnet_refresh_report_fixture()
-> SubnetCatalogRefreshReport {
    SubnetCatalogRefreshReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        catalog_path: ".icq/subnet-catalog/ic/catalog.json".to_string(),
        refresh_lock_path: ".icq/subnet-catalog/ic/refresh.lock".to_string(),
        output_path: None,
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        dry_run: false,
        wrote_catalog: true,
        replaced_existing_catalog: true,
        subnet_count: 2,
        routing_range_count: 3,
    }
}

pub(in crate::nns::topology::report::tests) fn dry_run_subnet_refresh_report_fixture()
-> SubnetCatalogRefreshReport {
    SubnetCatalogRefreshReport {
        dry_run: true,
        wrote_catalog: false,
        ..subnet_refresh_report_fixture()
    }
}

fn subnet_row(
    subnet_principal: &str,
    subnet_kind: SubnetKind,
    node_count: u32,
    range_count: usize,
) -> SubnetCatalogSubnetRow {
    SubnetCatalogSubnetRow {
        subnet_principal: subnet_principal.to_string(),
        subnet_kind,
        subnet_kind_source: ClassificationSource::Registry,
        subnet_specialization: SubnetSpecialization::None,
        subnet_specialization_source: ClassificationSource::Computed,
        geographic_scope: GeographicScope::Global,
        geographic_scope_source: ClassificationSource::Computed,
        subnet_label: subnet_kind.as_str().to_string(),
        subnet_label_source: ClassificationSource::Computed,
        node_count: Some(node_count),
        charges_apply_by_default: subnet_kind.charges_apply_by_default(),
        range_count,
        ranges_shown: 0,
        range_offset: 0,
        range_limit: 1,
        ranges: Vec::new(),
    }
}
