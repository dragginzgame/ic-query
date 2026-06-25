use super::{fixtures::*, *};

#[test]
fn node_provider_info_resolves_exact_principal() {
    let request = NnsNodeProviderInfoRequest {
        cache: test_cache_request(MAINNET_NETWORK, "info-exact"),
        source_endpoint: "https://icp-api.io".to_string(),
        input: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        now_unix_secs: 1_780_531_200,
    };
    let report = build_nns_node_provider_info_report_with_source(
        &request,
        &FixtureNodeProviderSource {
            node_providers: vec![MainnetNodeProvider {
                principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                node_count: Some(13),
                reward_account_hex: Some("abcd".to_string()),
            }],
        },
    )
    .expect("node provider info");

    assert_eq!(report.input, "ryjl3-tyaaa-aaaaa-aaaba-cai");
    assert_eq!(report.resolved_from, "node_provider_principal");
    assert_eq!(
        report.node_provider_principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
    assert_eq!(report.node_count, Some(13));
    assert_eq!(report.reward_account_hex.as_deref(), Some("abcd"));
}

#[test]
fn node_provider_info_resolves_unique_prefix() {
    let report = node_provider_report_fixture();

    let (provider, resolved_from) =
        resolve_node_provider(&report, "ryjl").expect("prefix resolves");

    assert_eq!(resolved_from, "node_provider_principal_prefix");
    assert_eq!(
        provider.node_provider_principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
}

#[test]
fn node_provider_info_rejects_ambiguous_prefix() {
    let report = NnsNodeProviderListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_provider_count: 2,
        node_providers: vec![
            NnsNodeProviderRow {
                node_provider_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                name: None,
                node_count: None,
                reward_account_hex: None,
            },
            NnsNodeProviderRow {
                node_provider_principal: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
                name: None,
                node_count: None,
                reward_account_hex: None,
            },
        ],
    };

    let err = resolve_node_provider(&report, "r").expect_err("ambiguous");

    assert!(matches!(
        err,
        NnsNodeProviderHostError::AmbiguousNodeProviderPrefix { prefix, matches }
            if prefix == "r" && matches.len() == 2
    ));
}
