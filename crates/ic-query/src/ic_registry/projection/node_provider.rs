use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, RegistryFetchError,
        transport::hex_bytes,
        wire::{GovernanceNodeProvider, ListNodeProvidersResponse},
    },
    nns::{
        MAINNET_GOVERNANCE_CANISTER_ID,
        node_provider::{
            NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION, NnsNodeProviderListReport,
            NnsNodeProviderRow,
        },
    },
};
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn node_provider_list_from_response(
    request: &MainnetRegistryFetchRequest,
    response: ListNodeProvidersResponse,
    node_counts: BTreeMap<String, u32>,
    registry_version: u64,
) -> Result<NnsNodeProviderListReport, RegistryFetchError> {
    let mut node_providers = response
        .node_providers
        .into_iter()
        .map(|node_provider| node_provider_from_governance(node_provider, &node_counts))
        .collect::<Result<Vec<_>, _>>()?;
    node_providers.sort_by(|left, right| {
        left.node_provider_principal
            .cmp(&right.node_provider_principal)
    });
    Ok(NnsNodeProviderListReport {
        schema_version: NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
        provenance: request.inventory_provenance(registry_version),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        node_providers,
    })
}

pub(in crate::ic_registry) fn node_provider_from_governance(
    node_provider: GovernanceNodeProvider,
    node_counts: &BTreeMap<String, u32>,
) -> Result<NnsNodeProviderRow, RegistryFetchError> {
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
    Ok(NnsNodeProviderRow {
        node_provider_principal: principal,
        name: None,
        node_count,
        reward_account_hex,
    })
}
