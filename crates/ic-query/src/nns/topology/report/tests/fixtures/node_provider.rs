use crate::nns::node_provider::{NnsNodeProviderListReport, NnsNodeProviderRow};

pub(in crate::nns::topology::report::tests) fn node_provider_report_fixture()
-> NnsNodeProviderListReport {
    NnsNodeProviderListReport {
        governance_source: crate::report::CollectionSourceRequest::from_unix_secs(
            "ic",
            "https://icp-api.io",
            1_780_531_200,
            "fixture",
        ),
        schema_version: 1,
        provenance: super::inventory_provenance(44, "2026-06-04T00:02:00Z"),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
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

    report.node_providers.push(NnsNodeProviderRow {
        node_provider_principal: "provider-z".to_string(),
        name: None,
        node_count: Some(1),
        reward_account_hex: None,
    });
    report
}
