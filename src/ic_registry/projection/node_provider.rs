use super::super::{
    MAINNET_GOVERNANCE_CANISTER_ID, MainnetNodeProvider, MainnetNodeProviderList,
    MainnetRegistryFetchRequest, RegistryFetchError,
    transport::hex_bytes,
    wire::{GovernanceNodeProvider, ListNodeProvidersResponse},
};
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn node_provider_list_from_response(
    request: &MainnetRegistryFetchRequest,
    response: ListNodeProvidersResponse,
    node_counts: BTreeMap<String, u32>,
    registry_version: u64,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    let mut node_providers = response
        .node_providers
        .into_iter()
        .map(|node_provider| node_provider_from_governance(node_provider, &node_counts))
        .collect::<Result<Vec<_>, _>>()?;
    node_providers.sort_by(|left, right| left.principal.cmp(&right.principal));
    Ok(MainnetNodeProviderList {
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        node_providers,
    })
}

pub(in crate::ic_registry) fn node_provider_from_governance(
    node_provider: GovernanceNodeProvider,
    node_counts: &BTreeMap<String, u32>,
) -> Result<MainnetNodeProvider, RegistryFetchError> {
    let principal = node_provider
        .id
        .ok_or(RegistryFetchError::MissingField {
            field: "node_provider.id",
        })?
        .to_text();
    let reward_account_hex = node_provider
        .reward_account
        .map(|account| hex_bytes(&account.hash));
    let node_count = Some(node_counts.get(&principal).copied().unwrap_or(0));
    Ok(MainnetNodeProvider {
        principal,
        node_count,
        reward_account_hex,
    })
}
