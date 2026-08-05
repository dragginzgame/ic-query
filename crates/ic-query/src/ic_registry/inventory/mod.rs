#[cfg(feature = "nns-host")]
mod data_center;
mod fetch;
mod keys;

const INVENTORY_FETCH_CONCURRENCY: usize = 32;

#[cfg(feature = "nns-host")]
pub(super) use fetch::fetch_node_provider_node_counts;
pub(super) use fetch::fetch_registry_relation_inventory;
