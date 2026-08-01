mod agent;
#[cfg(feature = "host")]
mod governance;
#[cfg(feature = "host")]
mod nodes;
#[cfg(feature = "host")]
mod registry;
#[cfg(feature = "host")]
mod relation_inventory;
mod subnet_catalog;
#[cfg(feature = "host")]
mod subnet_topology;

#[cfg(feature = "host")]
pub(super) use governance::fetch_mainnet_node_provider_list_async;
#[cfg(feature = "host")]
pub(super) use nodes::{
    fetch_mainnet_data_center_list_async, fetch_mainnet_node_list_async,
    fetch_mainnet_node_operator_list_async,
};
#[cfg(feature = "host")]
pub(super) use registry::fetch_mainnet_registry_version_async;
pub(super) use subnet_catalog::fetch_mainnet_subnet_catalog_async;
#[cfg(feature = "host")]
pub(super) use subnet_topology::fetch_mainnet_subnet_topology_async;
