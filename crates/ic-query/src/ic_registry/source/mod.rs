mod agent;
#[cfg(feature = "nns-host")]
mod governance;
#[cfg(feature = "nns-host")]
mod nodes;
#[cfg(feature = "nns-host")]
mod registry;
#[cfg(feature = "nns-topology-host")]
mod relation_inventory;
mod subnet_catalog;
#[cfg(feature = "nns-topology-host")]
mod subnet_topology;

#[cfg(feature = "nns-host")]
pub(super) use governance::fetch_mainnet_node_provider_list_async;
#[cfg(feature = "nns-host")]
pub(super) use nodes::{
    fetch_mainnet_data_center_list_async, fetch_mainnet_node_list_async,
    fetch_mainnet_node_operator_list_async,
};
#[cfg(feature = "nns-host")]
pub(super) use registry::{
    fetch_mainnet_certified_registry_delta_batch_async, fetch_mainnet_registry_version_async,
};
pub(super) use subnet_catalog::fetch_mainnet_subnet_catalog_async;
#[cfg(feature = "nns-topology-host")]
pub(super) use subnet_topology::fetch_mainnet_subnet_topology_async;
