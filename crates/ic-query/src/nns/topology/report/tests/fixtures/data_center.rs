use crate::{
    nns::data_center::report::{
        NnsDataCenterListReport, NnsDataCenterRefreshReport, NnsDataCenterRow,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};

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
