use super::NnsRegistryHostError;
use crate::{
    ic_registry::fetch_mainnet_registry_version,
    nns::{
        LiveNnsSource, registry::NnsRegistryVersionData, source::mainnet_registry_fetch_request,
    },
    report::CollectionSourceRequest,
};

///
/// NnsRegistrySource
///
/// Source contract for fetching NNS registry version data.
///

pub trait NnsRegistrySource: Sync {
    fn fetch_registry_version<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsRegistryVersionData, NnsRegistryHostError>>;
}

impl NnsRegistrySource for LiveNnsSource {
    fn fetch_registry_version<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<NnsRegistryVersionData, NnsRegistryHostError>> {
        Box::pin(async move {
            let fetch_request = mainnet_registry_fetch_request(request, |network| {
                NnsRegistryHostError::UnsupportedNetwork { network }
            })?;
            Ok(fetch_mainnet_registry_version(&fetch_request).await?)
        })
    }
}
