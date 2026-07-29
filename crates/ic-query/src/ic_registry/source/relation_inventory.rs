//! Module: ic_registry::source::relation_inventory
//!
//! Responsibility: collect one versioned Registry relation inventory.
//! Does not own: report projection, governance enrichment, or cache policy.
//! Boundary: resolves the Registry version once before every relation lookup.

use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::ic_registry::{
    MainnetRegistryFetchRequest, RegistryFetchError,
    inventory::fetch_registry_relation_inventory,
    relations::{RegistryRelationInventory, RegistryRelationInventoryScope},
    transport::get_latest_version,
};

///
/// MainnetRegistryRelationSnapshot
///
/// Required Registry relation records collected from one exact Registry version.
///

pub(super) struct MainnetRegistryRelationSnapshot {
    pub(super) registry_version: u64,
    pub(super) inventory: RegistryRelationInventory,
}

pub(super) async fn fetch_mainnet_registry_relation_snapshot(
    request: &MainnetRegistryFetchRequest,
    scope: RegistryRelationInventoryScope,
) -> Result<MainnetRegistryRelationSnapshot, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let inventory =
        fetch_registry_relation_inventory(&agent, &registry_canister, registry_version, scope)
            .await?;
    Ok(MainnetRegistryRelationSnapshot {
        registry_version,
        inventory,
    })
}
