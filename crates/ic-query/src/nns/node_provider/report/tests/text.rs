use super::{fixtures::*, *};

#[test]
fn node_provider_text_keeps_table_narrow() {
    let report = NnsNodeProviderListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_provider_count: 1,
        node_providers: vec![NnsNodeProviderRow {
            node_provider_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            name: Some("DFINITY".to_string()),
            node_count: Some(13),
            reward_account_hex: Some("abcd".to_string()),
        }],
    };

    let text = nns_node_provider_list_report_text(&report);

    assert!(text.contains("node_providers: ic count 1"));
    assert!(text.contains("NODE_PROVIDER"));
    assert!(text.contains("ryjl3"));
    assert!(text.contains("13"));
    assert!(!text.contains("NAME"));
    assert!(!text.contains("DFINITY"));
    assert!(!text.contains("ryjl3-tyaaa-aaaaa-aaaba-cai"));
    assert!(!text.contains("abcd"));
}

#[test]
fn node_provider_verbose_text_keeps_full_metadata() {
    let report = node_provider_report_fixture();

    let text = nns_node_provider_list_report_verbose_text(&report);

    assert!(text.contains("source_endpoint: https://icp-api.io"));
    assert!(text.contains("ryjl3-tyaaa-aaaaa-aaaba-cai"));
    assert!(text.contains("abcd"));
    assert!(text.contains("42"));
    assert!(text.contains("FETCHED_AT"));
    assert!(!text.contains("NAME"));
    assert!(!text.contains("DFINITY"));
}

#[test]
fn node_provider_info_text_renders_detail_lines() {
    let report = NnsNodeProviderInfoReport {
        schema_version: 1,
        input: "ryjl".to_string(),
        resolved_from: "node_provider_principal_prefix".to_string(),
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_provider_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        name: None,
        node_count: None,
        reward_account_hex: Some("abcd".to_string()),
    };

    let text = nns_node_provider_info_report_text(&report);

    assert!(text.contains("resolved_from: node_provider_principal_prefix"));
    assert!(text.contains("node_provider_principal: ryjl3-tyaaa-aaaaa-aaaba-cai"));
    assert!(!text.contains("name:"));
    assert!(text.contains("node_count: unknown"));
    assert!(text.contains("reward_account_hex: abcd"));
    assert!(text.contains("registry_version: 42"));
}
