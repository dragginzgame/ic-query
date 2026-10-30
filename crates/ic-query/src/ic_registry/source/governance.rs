use super::agent::{mainnet_agent, mainnet_governance_canister, mainnet_registry_canister};
use crate::candid_call::{encode_request, query_encoded};
use crate::ic_registry::{
    MainnetNodeProviderList, MainnetRegistryFetchRequest, RegistryFetchError,
    inventory::fetch_node_provider_node_counts, projection::node_provider_list_from_response,
    transport::get_latest_version, wire::ListNodeProvidersResponse,
};

crate::candid_call::impl_candid_call_error!(RegistryFetchError);

pub async fn fetch_mainnet_node_provider_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let governance_canister = mainnet_governance_canister()?;
    let arg = encode_request::<_, RegistryFetchError>((), "list_node_providers")?;
    let response = query_encoded::<ListNodeProvidersResponse, RegistryFetchError>(
        &agent,
        &governance_canister,
        "list_node_providers",
        "ListNodeProvidersResponse",
        arg,
    )
    .await?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let node_counts =
        fetch_node_provider_node_counts(&agent, &registry_canister, registry_version).await?;
    node_provider_list_from_response(request, response, node_counts, registry_version)
}
