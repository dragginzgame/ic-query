use super::{fixtures::*, *};

#[test]
fn governance_node_provider_response_converts_to_domain_structs() {
    let request = registry_fetch_request();
    let node_counts = BTreeMap::from([("aaaaa-aa".to_string(), 2)]);
    let response = ListNodeProvidersResponse {
        node_providers: vec![
            governance_node_provider("ryjl3-tyaaa-aaaaa-aaaba-cai", None),
            governance_node_provider("aaaaa-aa", Some(vec![0xab, 0xcd])),
        ],
    };

    let list = node_provider_list_from_response(&request, response, node_counts, 42)
        .expect("node providers");

    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.governance_canister_id, MAINNET_GOVERNANCE_CANISTER_ID);
    assert_eq!(list.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(list.registry_version, 42);
    assert_eq!(list.node_providers.len(), 2);
    assert_eq!(list.node_providers[0].principal, "aaaaa-aa");
    assert_eq!(list.node_providers[0].node_count, Some(2));
    assert_eq!(
        list.node_providers[0].reward_account_hex.as_deref(),
        Some("abcd")
    );
    assert_eq!(
        list.node_providers[1].principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
    assert_eq!(list.node_providers[1].node_count, Some(0));
    assert_eq!(list.node_providers[1].reward_account_hex, None);
}

#[test]
fn governance_node_provider_requires_principal() {
    let err = node_provider_from_governance(
        GovernanceNodeProvider {
            id: None,
            reward_account: None,
        },
        &BTreeMap::new(),
    )
    .expect_err("missing principal");

    assert!(matches!(
        err,
        RegistryFetchError::MissingField { field } if field == "node_provider.id"
    ));
}
