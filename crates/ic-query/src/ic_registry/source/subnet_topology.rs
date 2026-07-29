use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::ic_registry::{
    MainnetRegistryFetchRequest, MainnetSubnetTopology, RegistryFetchError,
    inventory::fetch_registry_relation_inventory, projection::subnet_topology_from_inventory,
    relations::RegistryRelationInventoryScope, transport::get_latest_version,
};

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_topology_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetSubnetTopology, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let inventory = fetch_registry_relation_inventory(
        &agent,
        &registry_canister,
        registry_version,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    subnet_topology_from_inventory(request, &inventory, registry_version)
}
