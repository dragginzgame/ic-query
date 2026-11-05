//! Project every NNS inventory view from one pinned relation acquisition.

use super::relation_inventory::fetch_mainnet_registry_relation_snapshot;
use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, RegistryFetchError,
        projection::{
            data_center_list_from_inventory, node_list_from_inventory,
            node_operator_list_from_inventory,
        },
        relations::RegistryRelationInventoryScope,
    },
    nns::NnsRegistryInventorySnapshot,
};

pub async fn fetch_mainnet_registry_inventory(
    request: &MainnetRegistryFetchRequest,
) -> Result<NnsRegistryInventorySnapshot, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::WithDataCenters,
    )
    .await?;
    Ok(NnsRegistryInventorySnapshot {
        schema_version: 1,
        provenance: request.inventory_provenance(snapshot.registry_version),
        nodes: node_list_from_inventory(&snapshot.inventory)?,
        node_operators: node_operator_list_from_inventory(&snapshot.inventory)?,
        data_centers: data_center_list_from_inventory(&snapshot.inventory)?,
    })
}
