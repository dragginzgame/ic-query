use super::{NnsInventoryError, NnsNodeProviderMetadata};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::fetch_mainnet_node_provider_metadata,
    nns::{LiveNnsSource, source::mainnet_registry_fetch_request},
};

///
/// NnsNodeProviderSource
///
/// Governance metadata capability paired with the independent Registry inventory source.
///

pub trait NnsNodeProviderSource: crate::nns::NnsRegistryInventorySource {
    /// Fetch provider identities and reward accounts without claiming Registry node counts.
    fn fetch_node_provider_metadata<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderMetadata, NnsInventoryError>>;
}

impl NnsNodeProviderSource for LiveNnsSource {
    fn fetch_node_provider_metadata<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderMetadata, NnsInventoryError>> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsInventoryError::UnsupportedNetwork {
                    collection: super::COLLECTION,
                    network,
                }
            })?;
            fetch_mainnet_node_provider_metadata(&fetch_request)
                .await
                .map_err(|source| NnsInventoryError::Registry {
                    collection: super::COLLECTION,
                    source,
                })
        })
    }
}
