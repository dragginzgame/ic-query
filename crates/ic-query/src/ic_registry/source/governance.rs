use super::agent::{mainnet_agent, mainnet_governance_canister, mainnet_registry_canister};
use crate::ic_registry::{
    MainnetNodeProviderList, MainnetRegistryFetchRequest, RegistryFetchError,
    inventory::fetch_node_provider_node_counts, projection::node_provider_list_from_response,
    transport::get_latest_version, wire::ListNodeProvidersResponse,
};
use candid::{Decode, Encode};

pub(in crate::ic_registry) async fn fetch_mainnet_node_provider_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let governance_canister = mainnet_governance_canister()?;
    let arg = Encode!().map_err(|err| RegistryFetchError::CandidEncode {
        message: "list_node_providers",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "list_node_providers")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| RegistryFetchError::AgentCall {
            method: "list_node_providers",
            reason: err.to_string(),
        })?;
    let response = Decode!(&bytes, ListNodeProvidersResponse).map_err(|err| {
        RegistryFetchError::CandidDecode {
            message: "ListNodeProvidersResponse",
            reason: err.to_string(),
        }
    })?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let node_counts =
        fetch_node_provider_node_counts(&agent, &registry_canister, registry_version).await?;
    node_provider_list_from_response(request, response, node_counts, registry_version)
}
