use super::relation_inventory::fetch_mainnet_registry_relation_snapshot;
use crate::ic_registry::{
    MainnetDataCenterList, MainnetNodeList, MainnetNodeOperatorList, MainnetRegistryFetchRequest,
    RegistryFetchError,
    projection::{
        data_center_list_from_inventory, node_list_from_inventory,
        node_operator_list_from_inventory,
    },
    relations::RegistryRelationInventoryScope,
};

pub async fn fetch_mainnet_node_operator_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeOperatorList, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    node_operator_list_from_inventory(request, snapshot.inventory, snapshot.registry_version)
}

pub async fn fetch_mainnet_node_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeList, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    node_list_from_inventory(request, snapshot.inventory, snapshot.registry_version)
}

pub async fn fetch_mainnet_data_center_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::WithDataCenters,
    )
    .await?;
    data_center_list_from_inventory(request, snapshot.inventory, snapshot.registry_version)
}
