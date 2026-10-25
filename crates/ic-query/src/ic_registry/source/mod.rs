mod agent;
#[cfg(feature = "nns-host")]
mod governance;
#[cfg(feature = "nns-host")]
mod nodes;
#[cfg(feature = "certified-subnet-catalog-host")]
mod registry;
#[cfg(feature = "nns-topology-host")]
mod relation_inventory;
mod subnet_catalog;
#[cfg(feature = "nns-topology-host")]
mod subnet_topology;

#[cfg(feature = "nns-host")]
pub use governance::fetch_mainnet_node_provider_list;
#[cfg(feature = "nns-host")]
pub use nodes::{
    fetch_mainnet_data_center_list, fetch_mainnet_node_list, fetch_mainnet_node_operator_list,
};
#[cfg(feature = "certified-subnet-catalog-host")]
pub use registry::{
    fetch_mainnet_certified_registry_delta_batch_async, fetch_mainnet_registry_version,
};
pub use subnet_catalog::fetch_mainnet_subnet_catalog_async;
#[cfg(feature = "nns-topology-host")]
pub use subnet_topology::fetch_mainnet_subnet_topology;

pub use subnet_catalog::fetch_with_acquisition;
