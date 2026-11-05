use super::{fixtures::*, *};
use crate::nns::MAINNET_GOVERNANCE_CANISTER_ID;

#[test]
fn governance_node_provider_response_converts_to_domain_structs() {
    let request = registry_fetch_request();
    let response = ListNodeProvidersResponse {
        node_providers: vec![
            governance_node_provider("ryjl3-tyaaa-aaaaa-aaaba-cai", None),
            governance_node_provider("aaaaa-aa", Some(vec![0xab, 0xcd])),
        ],
    };

    let list = node_provider_metadata_from_response(&request, response).expect("node providers");

    assert_eq!(list.source.network, MAINNET_NETWORK);
    assert_eq!(list.governance_canister_id, MAINNET_GOVERNANCE_CANISTER_ID);
    assert_eq!(list.node_providers.len(), 2);
    assert_eq!(list.node_providers[0].node_provider_principal, "aaaaa-aa");
    assert_eq!(list.node_providers[0].node_count, None);
    assert_eq!(
        list.node_providers[0].reward_account_hex.as_deref(),
        Some("abcd")
    );
    assert_eq!(
        list.node_providers[1].node_provider_principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
    assert_eq!(list.node_providers[1].node_count, None);
    assert_eq!(list.node_providers[1].reward_account_hex, None);
}

#[test]
fn governance_node_provider_requires_principal() {
    let err = node_provider_from_governance(GovernanceNodeProvider {
        id: None,
        reward_account: None,
    })
    .expect_err("missing principal");

    assert!(matches!(
        err,
        RegistryFetchError::MissingField { field } if field == "node_provider.id"
    ));
}
