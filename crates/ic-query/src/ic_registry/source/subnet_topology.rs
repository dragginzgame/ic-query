use super::relation_inventory::fetch_mainnet_registry_relation_snapshot;
use crate::ic_registry::{
    MainnetRegistryFetchRequest, RegistryFetchError, projection::subnet_topology_from_inventory,
    relations::RegistryRelationInventoryScope,
};

pub async fn fetch_mainnet_subnet_topology(
    request: &MainnetRegistryFetchRequest,
) -> Result<crate::nns::topology::NnsSubnetTopologyReport, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    subnet_topology_from_inventory(request, &snapshot.inventory, snapshot.registry_version)
}
