use super::error::NnsRegistryHostError;
use crate::ic_registry::{
    MainnetRegistryFetchRequest, MainnetRegistryVersion, fetch_mainnet_registry_version,
};

///
/// NnsRegistrySourceRequest
///
/// Source request settings for fetching the mainnet NNS registry version.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistrySourceRequest {
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl NnsRegistrySourceRequest {
    #[must_use]
    pub fn new(
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

///
/// NnsRegistryVersionData
///
/// Source-layer NNS registry version result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsRegistryVersionData {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
}

impl From<MainnetRegistryVersion> for NnsRegistryVersionData {
    fn from(version: MainnetRegistryVersion) -> Self {
        Self {
            network: version.network,
            registry_canister_id: version.registry_canister_id,
            registry_version: version.registry_version,
            fetched_at: version.fetched_at,
            fetched_by: version.fetched_by,
            source_endpoint: version.source_endpoint,
        }
    }
}

///
/// NnsRegistrySource
///
/// Source contract for fetching NNS registry version data.
///
pub trait NnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsRegistrySourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError>;
}

///
/// LiveNnsRegistrySource
///
/// Source implementation backed by live NNS registry calls.
///

pub struct LiveNnsRegistrySource;

impl NnsRegistrySource for LiveNnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsRegistrySourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        let request = MainnetRegistryFetchRequest {
            endpoint: request.endpoint.clone(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
        };
        Ok(fetch_mainnet_registry_version(&request)?.into())
    }
}
