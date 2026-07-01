use super::SubnetCatalogHostError;
use crate::{
    ic_registry::{MainnetRegistryFetchRequest, fetch_mainnet_subnet_catalog},
    subnet_catalog::SubnetCatalog,
};

///
/// SubnetCatalogSourceRequest
///
/// Source request settings for fetching one complete subnet catalog snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogSourceRequest {
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl SubnetCatalogSourceRequest {
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
/// SubnetCatalogSource
///
/// Source contract for fetching complete subnet catalog snapshots.
///

pub trait SubnetCatalogSource {
    fn fetch_catalog(
        &self,
        request: &SubnetCatalogSourceRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError>;
}

///
/// LiveNnsRegistryRefreshSource
///
/// Source implementation backed by live NNS registry calls.
///

pub struct LiveNnsRegistryRefreshSource;

impl SubnetCatalogSource for LiveNnsRegistryRefreshSource {
    fn fetch_catalog(
        &self,
        request: &SubnetCatalogSourceRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError> {
        let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
        fetch_request.endpoint.clone_from(&request.endpoint);
        fetch_request.fetched_by.clone_from(&request.fetched_by);
        Ok(fetch_mainnet_subnet_catalog(&fetch_request)?)
    }
}
