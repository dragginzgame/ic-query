use super::SubnetCatalogHostError;
use crate::{
    ic_registry::{fetch_mainnet_subnet_catalog, fetch_mainnet_subnet_catalog_async},
    nns::{LiveNnsSource, NnsSourceRequest, source::mainnet_registry_fetch_request},
    subnet_catalog::RawSubnetCatalog,
};

/// Fetch one live mainnet catalog without creating a Tokio runtime or helper thread.
pub async fn fetch_subnet_catalog_async(
    request: &NnsSourceRequest,
) -> Result<RawSubnetCatalog, SubnetCatalogHostError> {
    let fetch_request = mainnet_registry_fetch_request(request, |network| {
        SubnetCatalogHostError::UnsupportedNetwork { network }
    })?;
    Ok(fetch_mainnet_subnet_catalog_async(&fetch_request).await?)
}

///
/// SubnetCatalogSource
///
/// Source contract for fetching complete subnet catalog snapshots.
///

pub trait SubnetCatalogSource {
    fn fetch_catalog(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<RawSubnetCatalog, SubnetCatalogHostError>;
}

impl SubnetCatalogSource for LiveNnsSource {
    fn fetch_catalog(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<RawSubnetCatalog, SubnetCatalogHostError> {
        let fetch_request = mainnet_registry_fetch_request(request, |network| {
            SubnetCatalogHostError::UnsupportedNetwork { network }
        })?;
        Ok(fetch_mainnet_subnet_catalog(&fetch_request)?)
    }
}
