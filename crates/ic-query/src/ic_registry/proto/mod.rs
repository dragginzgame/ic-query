mod id;
#[cfg(feature = "nns-topology-host")]
mod node;
mod registry;
mod routing;
mod subnet;

#[cfg(all(test, feature = "subnet-catalog-host"))]
pub use id::PrincipalId;
pub use id::{CanisterId, SubnetId};
#[cfg(feature = "nns-host")]
pub use node::DataCenterRecord;
#[cfg(all(test, feature = "nns-host"))]
pub use node::Gps;
#[cfg(feature = "nns-topology-host")]
pub use node::{NodeOperatorRecord, NodeRecord};
#[cfg(all(test, feature = "nns-host"))]
pub use registry::RegistryError;
#[cfg(not(feature = "certified-subnet-catalog-host"))]
pub(in crate::ic_registry) use registry::RegistryGetChangesSinceRequest;
#[cfg(all(test, feature = "certified-subnet-catalog-host"))]
pub use registry::RegistryPrecondition;
#[cfg(feature = "certified-subnet-catalog-host")]
pub use registry::{
    HighCapacityRegistryAtomicMutateRequest, HighCapacityRegistryMutation,
    RegistryCertifiedResponse, RegistryGetChangesSinceRequest, RegistryMixedHashTree,
    RegistryMutationType, high_capacity_registry_mutation, registry_mixed_hash_tree,
};
#[cfg(test)]
pub(in crate::ic_registry) use registry::{HighCapacityRegistryDelta, HighCapacityRegistryValue};
pub(in crate::ic_registry) use registry::{
    HighCapacityRegistryGetChangesSinceResponse, high_capacity_registry_value,
};
pub use registry::{
    LargeValueChunkKeys, RegistryErrorCode, RegistryGetLatestVersionResponse,
    RegistryGetValueRequest, RegistryGetValueResponse, UInt64Value, registry_get_value_response,
};
pub use routing::RoutingTable;
#[cfg(all(test, feature = "subnet-catalog-host"))]
pub use routing::{CanisterIdRange, RoutingTableEntry};
#[cfg(all(
    test,
    any(
        feature = "certified-subnet-catalog-host",
        feature = "nns-topology-host"
    )
))]
pub use subnet::SubnetType;
pub use subnet::{SubnetListRecord, SubnetRecord};
