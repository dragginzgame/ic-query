use crate::nns::node_operator::{NnsNodeOperatorListReport, NnsNodeOperatorRow};

pub(in crate::nns::topology::report::tests) fn node_operator_report_fixture()
-> NnsNodeOperatorListReport {
    NnsNodeOperatorListReport {
        schema_version: 1,
        provenance: super::inventory_provenance(45, "2026-06-04T00:03:00Z"),
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

    report.node_operators.push(NnsNodeOperatorRow {
        node_operator_principal: "operator-z".to_string(),
        node_provider_principal: "provider-z".to_string(),
        node_allowance: 1,
        data_center_id: "dc-z".to_string(),
        node_count: Some(1),
    });
    report
}
