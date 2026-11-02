use super::relation_inventory::fetch_mainnet_registry_relation_snapshot;
use crate::ic_registry::model::RegistryInventoryRows;
use crate::ic_registry::{
    MainnetRegistryFetchRequest, RegistryFetchError,
    projection::{
        data_center_list_from_inventory, node_list_from_inventory,
        node_operator_list_from_inventory,
    },
    relations::RegistryRelationInventoryScope,
};

pub async fn fetch_mainnet_node_operator_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<RegistryInventoryRows<crate::nns::node_operator::NnsNodeOperatorRow>, RegistryFetchError>
{
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    Ok((
        request.inventory_provenance(snapshot.registry_version),
        node_operator_list_from_inventory(snapshot.inventory)?,
    ))
}

pub async fn fetch_mainnet_node_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<RegistryInventoryRows<crate::nns::node::NnsNodeRow>, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    Ok((
        request.inventory_provenance(snapshot.registry_version),
        node_list_from_inventory(snapshot.inventory)?,
    ))
}

pub async fn fetch_mainnet_data_center_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<RegistryInventoryRows<crate::nns::data_center::NnsDataCenterRow>, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::WithDataCenters,
    )
    .await?;
    Ok((
        request.inventory_provenance(snapshot.registry_version),
        data_center_list_from_inventory(snapshot.inventory)?,
    ))
}
