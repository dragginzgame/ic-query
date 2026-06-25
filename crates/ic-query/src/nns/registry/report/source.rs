use super::error::NnsRegistryHostError;
use crate::ic_registry::{
    MainnetRegistryFetchRequest, MainnetRegistryVersion, fetch_mainnet_registry_version,
};

///
/// NnsRegistrySource
///
pub(super) trait NnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetRegistryVersion, NnsRegistryHostError>;
}

///
/// LiveNnsRegistrySource
///
pub(super) struct LiveNnsRegistrySource;

impl NnsRegistrySource for LiveNnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetRegistryVersion, NnsRegistryHostError> {
        Ok(fetch_mainnet_registry_version(request)?)
    }
}
