use super::SubnetCatalogHostError;
use crate::{
    ic_registry::{MainnetRegistryFetchRequest, fetch_mainnet_subnet_catalog},
    subnet_catalog::SubnetCatalog,
};

pub trait SubnetCatalogRefreshSource {
    fn fetch_catalog(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError>;
}

///
/// LiveNnsRegistryRefreshSource
///
pub struct LiveNnsRegistryRefreshSource;

impl SubnetCatalogRefreshSource for LiveNnsRegistryRefreshSource {
    fn fetch_catalog(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError> {
        Ok(fetch_mainnet_subnet_catalog(request)?)
    }
}
