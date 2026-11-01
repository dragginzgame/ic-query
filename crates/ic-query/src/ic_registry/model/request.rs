use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
#[cfg(feature = "certified-subnet-catalog-host")]
use crate::{
    nns::registry::NnsRegistryInventoryProvenance,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};

///
/// MainnetRegistryFetchRequest
///
/// Source settings for a mainnet registry fetch.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetRegistryFetchRequest {
    pub endpoint: String,
    /// Canonical UTC collection timestamp with second precision.
    pub fetched_at: String,
    pub fetched_by: String,
}

impl MainnetRegistryFetchRequest {
    #[must_use]
    pub fn new(fetched_at: String) -> Self {
        Self {
            endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            fetched_at,
            fetched_by: "ic-query".to_string(),
        }
    }
}

#[cfg(feature = "certified-subnet-catalog-host")]
impl MainnetRegistryFetchRequest {
    pub(in crate::ic_registry) fn inventory_provenance(
        &self,
        registry_version: u64,
    ) -> NnsRegistryInventoryProvenance {
        NnsRegistryInventoryProvenance {
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version,
            fetched_at: self.fetched_at.clone(),
            source_endpoint: self.endpoint.clone(),
            fetched_by: self.fetched_by.clone(),
        }
    }
}
