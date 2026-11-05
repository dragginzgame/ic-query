use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, RegistryFetchError,
        transport::hex_bytes,
        wire::{GovernanceNodeProvider, ListNodeProvidersResponse},
    },
    nns::{
        MAINNET_GOVERNANCE_CANISTER_ID,
        node_provider::{
            NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION, NnsNodeProviderMetadata,
            NnsNodeProviderRow,
        },
    },
};

pub(in crate::ic_registry) fn node_provider_metadata_from_response(
    request: &MainnetRegistryFetchRequest,
    response: ListNodeProvidersResponse,
) -> Result<NnsNodeProviderMetadata, RegistryFetchError> {
    let mut node_providers = response
        .node_providers
        .into_iter()
        .map(node_provider_from_governance)
        .collect::<Result<Vec<_>, _>>()?;
    node_providers.sort_by(|left, right| {
        left.node_provider_principal
            .cmp(&right.node_provider_principal)
    });
    Ok(NnsNodeProviderMetadata {
        schema_version: NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
        source: crate::report::CollectionSourceRequest::new(
            crate::subnet_catalog::MAINNET_NETWORK,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
        ),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        node_providers,
    })
}

pub(in crate::ic_registry) fn node_provider_from_governance(
    node_provider: GovernanceNodeProvider,
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
    Ok(NnsNodeProviderRow {
        node_provider_principal: principal,
        name: None,
        node_count: None,
        reward_account_hex,
    })
}
