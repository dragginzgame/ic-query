use super::SubnetCatalogHostError;
use crate::{
    ic_registry::fetch_mainnet_subnet_catalog,
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
    subnet_catalog::SubnetCatalog,
};

///
/// SubnetCatalogSource
///
/// Source contract for fetching complete subnet catalog snapshots.
///

pub trait SubnetCatalogSource {
    fn fetch_catalog(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError>;
}

impl SubnetCatalogSource for LiveNnsSource {
    fn fetch_catalog(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError> {
        let fetch_request = mainnet_registry_fetch_request(request, |network| {
            SubnetCatalogHostError::UnsupportedNetwork { network }
        })?;
        Ok(fetch_mainnet_subnet_catalog(&fetch_request)?)
    }
}
