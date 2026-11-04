use super::{NnsInventoryError, NnsNodeProviderListReport};
use crate::report::CollectionSourceRequest;
use crate::{
    ic_registry::fetch_mainnet_node_provider_list,
    nns::{LiveNnsSource, source::mainnet_registry_fetch_request},
};

///
/// NnsNodeProviderSource
///
/// Source contract for fetching complete NNS node-provider list reports.
///

pub trait NnsNodeProviderSource: Sync {
    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsInventoryError>>;
}

impl NnsNodeProviderSource for LiveNnsSource {
    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsInventoryError>> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsInventoryError::UnsupportedNetwork {
                    collection: super::COLLECTION,
                    network,
                }
            })?;
            fetch_mainnet_node_provider_list(&fetch_request)
                .await
                .map_err(|source| NnsInventoryError::Registry {
                    collection: super::COLLECTION,
                    source,
                })
        })
    }
}
