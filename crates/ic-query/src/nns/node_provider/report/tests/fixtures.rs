use super::*;
use crate::test_support::temp_dir;

pub(super) fn node_provider_report_fixture() -> NnsNodeProviderListReport {
    NnsNodeProviderListReport {
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
                node_provider_principal: "aaaaa-aa".to_string(),
                name: None,
                node_count: Some(3),
                reward_account_hex: None,
            },
            NnsNodeProviderRow {
                node_provider_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                name: Some("DFINITY".to_string()),
                node_count: Some(13),
                reward_account_hex: Some("abcd".to_string()),
            },
        ],
    }
}

pub(super) struct FixtureNodeProviderSource {
    pub(super) node_providers: Vec<MainnetNodeProvider>,
}

impl NnsNodeProviderSource for FixtureNodeProviderSource {
    fn fetch_node_providers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeProviderList, NnsNodeProviderHostError> {
        Ok(MainnetNodeProviderList {
            network: MAINNET_NETWORK.to_string(),
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: "test".to_string(),
            source_endpoint: request.endpoint.clone(),
            node_providers: self.node_providers.clone(),
        })
    }
}

pub(super) struct FailingNodeProviderSource;

impl NnsNodeProviderSource for FailingNodeProviderSource {
    fn fetch_node_providers(
        &self,
        _request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeProviderList, NnsNodeProviderHostError> {
        Err(NnsNodeProviderHostError::NodeProviderNotFound {
            input: "unexpected-live-fetch".to_string(),
        })
    }
}

pub(super) fn test_cache_request(network: &str, name: &str) -> NnsNodeProviderCacheRequest {
    NnsNodeProviderCacheRequest {
        icp_root: temp_dir(&format!("ic-query-nns-node-provider-{name}")),
        network: network.to_string(),
    }
}
