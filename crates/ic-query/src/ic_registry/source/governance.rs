use super::agent::{mainnet_agent, mainnet_governance_canister};
use crate::candid_call::{encode_request, query_encoded};
use crate::ic_registry::{
    MainnetRegistryFetchRequest, RegistryFetchError,
    projection::node_provider_metadata_from_response, wire::ListNodeProvidersResponse,
};

crate::candid_call::impl_candid_call_error!(RegistryFetchError);

pub async fn fetch_mainnet_node_provider_metadata(
    request: &MainnetRegistryFetchRequest,
) -> Result<crate::nns::node_provider::NnsNodeProviderMetadata, RegistryFetchError> {
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
    node_provider_metadata_from_response(request, response)
}
