//! Reusable Network Nervous System query families, models.

#[cfg(feature = "nns-host")]
#[macro_use]
mod macros;
pub mod data_center;
pub mod governance;
#[cfg(feature = "nns-host")]
mod governance_query;
#[cfg(feature = "nns-host")]
pub(crate) mod inventory;
#[cfg(feature = "nns-host")]
mod inventory_error;
#[cfg(feature = "nns-host")]
pub use inventory_error::{NnsInventoryCollection, NnsInventoryError};
mod inventory_request;
#[cfg(feature = "nns-host")]
mod leaf;
pub mod neuron;
pub mod node;
pub mod node_operator;
pub mod node_provider;
pub mod proposals;
pub mod registry;
#[cfg(feature = "subnet-catalog-host")]
pub(crate) mod source;
pub mod topology;

pub(crate) const MAINNET_GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

#[cfg(feature = "nns-host")]
pub use governance::collection::{
    NnsGovernanceCacheAdmission, NnsGovernanceCacheRequest, NnsGovernanceCacheStatusReport,
    NnsGovernanceCacheSummary, NnsGovernanceRefreshReport, NnsGovernanceRefreshRequest,
    NnsGovernanceValidCacheSummary,
};
#[cfg(feature = "nns-host")]
pub use inventory::{
    NnsInventoryRefreshReport, NnsRegistryInventorySnapshot, NnsRegistryInventorySource,
    nns_registry_inventory_cache_path, nns_registry_inventory_refresh_lock_path,
    refresh_nns_registry_inventory, refresh_nns_registry_inventory_with_source,
};
#[cfg(feature = "nns-host")]
pub use inventory_request::NnsInventoryRefreshRequest;
pub use inventory_request::{
    NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest,
};
#[cfg(feature = "subnet-catalog-host")]
pub use source::LiveNnsSource;

pub use governance::NnsGovernanceCollection;
