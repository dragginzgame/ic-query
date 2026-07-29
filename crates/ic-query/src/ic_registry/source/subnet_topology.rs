use super::relation_inventory::fetch_mainnet_registry_relation_snapshot;
use crate::ic_registry::{
    MainnetRegistryFetchRequest, MainnetSubnetTopology, RegistryFetchError,
    projection::subnet_topology_from_inventory, relations::RegistryRelationInventoryScope,
};

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_topology_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetSubnetTopology, RegistryFetchError> {
    let snapshot = fetch_mainnet_registry_relation_snapshot(
        request,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    subnet_topology_from_inventory(request, &snapshot.inventory, snapshot.registry_version)
}
