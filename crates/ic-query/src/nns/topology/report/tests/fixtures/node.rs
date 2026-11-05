use crate::{
    nns::node::{NnsNodeListReport, NnsNodeRow},
    subnet_catalog::SubnetKind,
};

pub(in crate::nns::topology::report::tests) fn node_report_fixture() -> NnsNodeListReport {
    NnsNodeListReport {
        schema_version: 1,
        provenance: super::inventory_provenance(43, "2026-06-04T00:01:00Z"),
        nodes: vec![
            node_row(
                "node-a",
                "operator-a",
                "provider-a",
                "dc1",
                SubnetKind::Application,
            ),
            node_row(
                "node-b",
                "operator-a",
                "provider-a",
                "dc1",
                SubnetKind::Application,
            ),
            node_row(
                "node-c",
                "operator-z",
                "provider-z",
                "dc-z",
                SubnetKind::System,
            ),
        ],
    }
}

fn node_row(
    node_principal: &str,
    node_operator_principal: &str,
    node_provider_principal: &str,
    data_center_id: &str,
    subnet_kind: SubnetKind,
) -> NnsNodeRow {
    NnsNodeRow {
        node_principal: node_principal.to_string(),
        node_operator_principal: node_operator_principal.to_string(),
        node_provider_principal: node_provider_principal.to_string(),
        subnet_principal: "subnet-a".to_string(),
        subnet_kind,
        data_center_id: data_center_id.to_string(),
    }
}
