use crate::{
    nns::data_center::report::{
        NnsDataCenterListReport, NnsDataCenterRefreshReport, NnsDataCenterRow,
    },
    nns::node::report::{NnsNodeListReport, NnsNodeRefreshReport, NnsNodeRow},
    nns::node_operator::report::{
        NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport, NnsNodeOperatorRow,
    },
    nns::node_provider::report::{
        NnsNodeProviderListReport, NnsNodeProviderRefreshReport, NnsNodeProviderRow,
    },
    subnet_catalog::{
        ClassificationSource, GeographicScope, MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID,
        SubnetCatalogListReport, SubnetCatalogRefreshReport, SubnetCatalogSubnetRow, SubnetKind,
        SubnetSpecialization,
    },
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

pub(in crate::nns::topology::report::tests) fn node_refresh_report_fixture() -> NnsNodeRefreshReport
{
    NnsNodeRefreshReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        cache_path: ".icq/node/ic/nodes.json".to_string(),
        refresh_lock_path: ".icq/node/ic/refresh.lock".to_string(),
        output_path: None,
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 43,
        fetched_at: "2026-06-04T00:01:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        dry_run: false,
        wrote_cache: true,
        replaced_existing_cache: false,
        node_count: 3,
    }
}

pub(in crate::nns::topology::report::tests) fn dry_run_node_refresh_report_fixture()
-> NnsNodeRefreshReport {
    NnsNodeRefreshReport {
        dry_run: true,
        wrote_cache: false,
        ..node_refresh_report_fixture()
    }
}

pub(in crate::nns::topology::report::tests) fn node_provider_refresh_report_fixture()
-> NnsNodeProviderRefreshReport {
    NnsNodeProviderRefreshReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        cache_path: ".icq/node-provider/ic/providers.json".to_string(),
        refresh_lock_path: ".icq/node-provider/ic/refresh.lock".to_string(),
        output_path: None,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 44,
        fetched_at: "2026-06-04T00:02:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        dry_run: false,
        wrote_cache: true,
        replaced_existing_cache: false,
        node_provider_count: 1,
    }
}

pub(in crate::nns::topology::report::tests) fn dry_run_node_provider_refresh_report_fixture()
-> NnsNodeProviderRefreshReport {
    NnsNodeProviderRefreshReport {
        dry_run: true,
        wrote_cache: false,
        ..node_provider_refresh_report_fixture()
    }
}

pub(in crate::nns::topology::report::tests) fn node_operator_refresh_report_fixture()
-> NnsNodeOperatorRefreshReport {
    NnsNodeOperatorRefreshReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        cache_path: ".icq/node-operator/ic/operators.json".to_string(),
        refresh_lock_path: ".icq/node-operator/ic/refresh.lock".to_string(),
        output_path: None,
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 45,
        fetched_at: "2026-06-04T00:03:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        dry_run: false,
        wrote_cache: true,
        replaced_existing_cache: false,
        node_operator_count: 2,
    }
}

pub(in crate::nns::topology::report::tests) fn dry_run_node_operator_refresh_report_fixture()
-> NnsNodeOperatorRefreshReport {
    NnsNodeOperatorRefreshReport {
        dry_run: true,
        wrote_cache: false,
        ..node_operator_refresh_report_fixture()
    }
}

pub(in crate::nns::topology::report::tests) fn data_center_refresh_report_fixture()
-> NnsDataCenterRefreshReport {
    NnsDataCenterRefreshReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        cache_path: ".icq/data-center/ic/data-centers.json".to_string(),
        refresh_lock_path: ".icq/data-center/ic/refresh.lock".to_string(),
        output_path: None,
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 46,
        fetched_at: "2026-06-04T00:04:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        dry_run: false,
        wrote_cache: true,
        replaced_existing_cache: false,
        data_center_count: 1,
    }
}

pub(in crate::nns::topology::report::tests) fn dry_run_data_center_refresh_report_fixture()
-> NnsDataCenterRefreshReport {
    NnsDataCenterRefreshReport {
        dry_run: true,
        wrote_cache: false,
        ..data_center_refresh_report_fixture()
    }
}

pub(in crate::nns::topology::report::tests) fn node_report_fixture() -> NnsNodeListReport {
    NnsNodeListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 43,
        fetched_at: "2026-06-04T00:01:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_count: 3,
        nodes: vec![
            node_row("node-a", "operator-a", "provider-a", "dc1", "application"),
            node_row("node-b", "operator-a", "provider-a", "dc1", "application"),
            node_row("node-c", "operator-z", "provider-z", "dc-z", "system"),
        ],
    }
}

pub(in crate::nns::topology::report::tests) fn node_provider_report_fixture()
-> NnsNodeProviderListReport {
    NnsNodeProviderListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 44,
        fetched_at: "2026-06-04T00:02:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_provider_count: 1,
        node_providers: vec![NnsNodeProviderRow {
            node_provider_principal: "provider-a".to_string(),
            name: None,
            node_count: Some(3),
            reward_account_hex: None,
        }],
    }
}

pub(in crate::nns::topology::report::tests) fn complete_node_provider_report_fixture()
-> NnsNodeProviderListReport {
    let mut report = node_provider_report_fixture();
    report.node_provider_count = 2;
    report.node_providers.push(NnsNodeProviderRow {
        node_provider_principal: "provider-z".to_string(),
        name: None,
        node_count: Some(1),
        reward_account_hex: None,
    });
    report
}

pub(in crate::nns::topology::report::tests) fn node_operator_report_fixture()
-> NnsNodeOperatorListReport {
    NnsNodeOperatorListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 45,
        fetched_at: "2026-06-04T00:03:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_operator_count: 2,
        node_operators: vec![
            NnsNodeOperatorRow {
                node_operator_principal: "operator-a".to_string(),
                node_provider_principal: "provider-a".to_string(),
                node_allowance: 1,
                data_center_id: "dc1".to_string(),
                node_count: Some(2),
            },
            NnsNodeOperatorRow {
                node_operator_principal: "operator-b".to_string(),
                node_provider_principal: "provider-z".to_string(),
                node_allowance: 1,
                data_center_id: "dc-z".to_string(),
                node_count: Some(1),
            },
        ],
    }
}

pub(in crate::nns::topology::report::tests) fn complete_node_operator_report_fixture()
-> NnsNodeOperatorListReport {
    let mut report = node_operator_report_fixture();
    report.node_operator_count = 3;
    report.node_operators.push(NnsNodeOperatorRow {
        node_operator_principal: "operator-z".to_string(),
        node_provider_principal: "provider-z".to_string(),
        node_allowance: 1,
        data_center_id: "dc-z".to_string(),
        node_count: Some(1),
    });
    report
}

pub(in crate::nns::topology::report::tests) fn data_center_report_fixture()
-> NnsDataCenterListReport {
    NnsDataCenterListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 46,
        fetched_at: "2026-06-04T00:04:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        data_center_count: 1,
        data_centers: vec![NnsDataCenterRow {
            data_center_id: "dc1".to_string(),
            region: "eu-west".to_string(),
            owner: "example".to_string(),
            latitude: None,
            longitude: None,
            node_operator_count: 2,
            node_provider_count: 1,
            node_count: 3,
        }],
    }
}

pub(in crate::nns::topology::report::tests) fn complete_data_center_report_fixture()
-> NnsDataCenterListReport {
    let mut report = data_center_report_fixture();
    report.data_center_count = 2;
    report.data_centers.push(NnsDataCenterRow {
        data_center_id: "dc-z".to_string(),
        region: "eu-west".to_string(),
        owner: "example".to_string(),
        latitude: None,
        longitude: None,
        node_operator_count: 1,
        node_provider_count: 1,
        node_count: 1,
    });
    report
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

fn node_row(
    node_principal: &str,
    node_operator_principal: &str,
    node_provider_principal: &str,
    data_center_id: &str,
    subnet_kind: &str,
) -> NnsNodeRow {
    NnsNodeRow {
        node_principal: node_principal.to_string(),
        node_operator_principal: node_operator_principal.to_string(),
        node_provider_principal: node_provider_principal.to_string(),
        subnet_principal: "subnet-a".to_string(),
        subnet_kind: subnet_kind.to_string(),
        data_center_id: data_center_id.to_string(),
    }
}
