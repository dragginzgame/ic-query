use crate::nns::NnsInventoryRefreshReport;
use crate::nns::data_center::{NnsDataCenterListReport, NnsDataCenterRow};

pub(in crate::nns::topology::report::tests) fn data_center_refresh_report_fixture()
-> NnsInventoryRefreshReport {
    NnsInventoryRefreshReport {
        schema_version: 1,
        collection: "data_centers".to_string(),
        provenance: super::inventory_provenance(46, "2026-06-04T00:04:00Z"),
        cache_path: "/cache/nns/ic/data-center/data-centers.json".to_string(),
        refresh_lock_path: "/cache/nns/ic/data-center/refresh.lock".to_string(),
        output_path: None,
        governance_canister_id: None,
        dry_run: false,
        wrote_cache: true,
        replaced_existing_cache: false,
        row_count: 1,
    }
}

pub(in crate::nns::topology::report::tests) fn dry_run_data_center_refresh_report_fixture()
-> NnsInventoryRefreshReport {
    NnsInventoryRefreshReport {
        dry_run: true,
        wrote_cache: false,
        ..data_center_refresh_report_fixture()
    }
}

pub(in crate::nns::topology::report::tests) fn data_center_report_fixture()
-> NnsDataCenterListReport {
    NnsDataCenterListReport {
        schema_version: 1,
        provenance: super::inventory_provenance(46, "2026-06-04T00:04:00Z"),
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
