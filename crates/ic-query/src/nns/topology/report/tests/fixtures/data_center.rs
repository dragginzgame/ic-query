use crate::nns::data_center::{NnsDataCenterListReport, NnsDataCenterRow};

pub(in crate::nns::topology::report::tests) fn data_center_report_fixture()
-> NnsDataCenterListReport {
    NnsDataCenterListReport {
        schema_version: 1,
        provenance: super::inventory_provenance(46, "2026-06-04T00:04:00Z"),
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
