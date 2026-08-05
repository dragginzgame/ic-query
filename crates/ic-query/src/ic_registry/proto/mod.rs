mod id;
#[cfg(feature = "nns-topology-host")]
mod node;
mod registry;
mod routing;
mod subnet;

#[cfg(all(test, feature = "host"))]
pub use id::PrincipalId;
pub use id::{CanisterId, SubnetId};
#[cfg(feature = "host")]
pub use node::DataCenterRecord;
#[cfg(all(test, feature = "host"))]
pub use node::Gps;
#[cfg(feature = "nns-topology-host")]
pub use node::{NodeOperatorRecord, NodeRecord};
#[cfg(all(test, feature = "host"))]
pub use registry::RegistryError;
pub use registry::{
    LargeValueChunkKeys, RegistryErrorCode, RegistryGetLatestVersionResponse,
    RegistryGetValueRequest, RegistryGetValueResponse, UInt64Value, registry_get_value_response,
};
pub use routing::RoutingTable;
#[cfg(all(test, feature = "host"))]
pub use routing::{CanisterIdRange, RoutingTableEntry};
#[cfg(all(test, feature = "nns-topology-host"))]
pub use subnet::SubnetType;
pub use subnet::{SubnetListRecord, SubnetRecord};
