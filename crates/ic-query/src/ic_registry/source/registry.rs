use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, MainnetRegistryVersion, RegistryFetchError,
        transport::get_latest_version,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};

pub(in crate::ic_registry) async fn fetch_mainnet_registry_version_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetRegistryVersion, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    Ok(MainnetRegistryVersion {
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
    })
}
