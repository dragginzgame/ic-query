#[cfg(feature = "nns-host")]
mod node_provider;
mod request;
#[cfg(feature = "nns-topology-host")]
mod subnet_topology;

#[cfg(feature = "nns-host")]
pub use node_provider::{MainnetNodeProvider, MainnetNodeProviderList};
pub use request::MainnetRegistryFetchRequest;
#[cfg(feature = "nns-topology-host")]
pub use subnet_topology::{
    MainnetSubnetTopology, MainnetSubnetTopologyNodeProvider, MainnetSubnetTopologySubnet,
};

/// Rows and Registry acquisition evidence passed to the report owner.
#[cfg(feature = "nns-host")]
pub(in crate::ic_registry) type RegistryInventoryRows<Row> = (
    crate::nns::registry::NnsRegistryInventoryProvenance,
    Vec<Row>,
);
