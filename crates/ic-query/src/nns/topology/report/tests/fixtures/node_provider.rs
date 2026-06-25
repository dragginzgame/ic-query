use crate::{
    nns::node_provider::report::{
        NnsNodeProviderListReport, NnsNodeProviderRefreshReport, NnsNodeProviderRow,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};

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
