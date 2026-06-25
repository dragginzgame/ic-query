use crate::{
    nns::node::report::{NnsNodeListReport, NnsNodeRefreshReport, NnsNodeRow},
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};

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
