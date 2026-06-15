use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::ic_registry::{
    MainnetDataCenterList, MainnetNodeList, MainnetNodeOperatorList, MainnetRegistryFetchRequest,
    RegistryFetchError,
    inventory::fetch_registry_relation_inventory,
    projection::{
        data_center_list_from_inventory, node_list_from_inventory,
        node_operator_list_from_inventory,
    },
    relations::RegistryRelationInventoryScope,
    transport::get_latest_version,
};

pub(in crate::ic_registry) async fn fetch_mainnet_node_operator_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeOperatorList, RegistryFetchError> {
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
    node_operator_list_from_inventory(request, inventory, registry_version)
}

pub(in crate::ic_registry) async fn fetch_mainnet_node_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeList, RegistryFetchError> {
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
    node_list_from_inventory(request, inventory, registry_version)
}

pub(in crate::ic_registry) async fn fetch_mainnet_data_center_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let inventory = fetch_registry_relation_inventory(
        &agent,
        &registry_canister,
        registry_version,
        RegistryRelationInventoryScope::WithDataCenters,
    )
    .await?;
    data_center_list_from_inventory(request, inventory, registry_version)
}
