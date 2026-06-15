use crate::{
    nns::node_operator::report::{
        NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport, NnsNodeOperatorRow,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};

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
