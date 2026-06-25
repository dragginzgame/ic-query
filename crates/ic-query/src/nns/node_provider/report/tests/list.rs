use super::{fixtures::*, *};

#[test]
fn node_provider_report_uses_live_governance_source() {
    let request = NnsNodeProviderListRequest {
        cache: test_cache_request(MAINNET_NETWORK, "uses-live-source"),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };
    let report = build_nns_node_provider_list_report_with_source(
        &request,
        &FixtureNodeProviderSource {
            node_providers: vec![
                MainnetNodeProvider {
                    principal: "aaaaa-aa".to_string(),
                    node_count: Some(3),
                    reward_account_hex: Some("abcd".to_string()),
                },
                MainnetNodeProvider {
                    principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                    node_count: None,
                    reward_account_hex: None,
                },
            ],
        },
    )
    .expect("node provider report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(
        report.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(report.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(report.registry_version, 42);
    assert_eq!(report.fetched_at, "2026-06-04T00:00:00Z");
    assert_eq!(report.node_provider_count, 2);
    assert_eq!(report.node_providers[0].node_provider_principal, "aaaaa-aa");
    assert_eq!(report.node_providers[0].name, None);
    assert_eq!(report.node_providers[0].node_count, Some(3));
    assert_eq!(
        report.node_providers[0].reward_account_hex.as_deref(),
        Some("abcd")
    );
}

#[test]
fn node_provider_list_rejects_local_network() {
    let request = NnsNodeProviderListRequest {
        cache: test_cache_request("local", "local-rejected"),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1,
    };

    let err = build_nns_node_provider_list_report_with_source(
        &request,
        &FixtureNodeProviderSource {
            node_providers: Vec::new(),
        },
    )
    .expect_err("local rejected");

    assert!(err.to_string().contains("supports only the mainnet `ic`"));
}
