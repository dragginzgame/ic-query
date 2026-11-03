use crate::nns::NnsInventoryRefreshReport;
use crate::nns::node_provider::{NnsNodeProviderListReport, NnsNodeProviderRow};

pub(in crate::nns::topology::report::tests) fn node_provider_refresh_report_fixture()
-> NnsInventoryRefreshReport {
    NnsInventoryRefreshReport {
        schema_version: 1,
        collection: "node_providers".to_string(),
        provenance: super::inventory_provenance(44, "2026-06-04T00:02:00Z"),
        cache_path: "/cache/nns/ic/node-provider/providers.json".to_string(),
        refresh_lock_path: "/cache/nns/ic/node-provider/refresh.lock".to_string(),
        output_path: None,
        governance_canister_id: Some("rrkah-fqaaa-aaaaa-aaaaq-cai".to_string()),
        dry_run: false,
        wrote_cache: true,
        replaced_existing_cache: false,
        row_count: 1,
    }
}

pub(in crate::nns::topology::report::tests) fn dry_run_node_provider_refresh_report_fixture()
-> NnsInventoryRefreshReport {
    NnsInventoryRefreshReport {
        dry_run: true,
        wrote_cache: false,
        ..node_provider_refresh_report_fixture()
    }
}

pub(in crate::nns::topology::report::tests) fn node_provider_report_fixture()
-> NnsNodeProviderListReport {
    NnsNodeProviderListReport {
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
