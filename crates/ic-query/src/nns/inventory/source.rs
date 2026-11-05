//! Source capability for one complete Registry inventory.

use super::NnsRegistryInventorySnapshot;
use crate::{
    nns::{
        LiveNnsSource, NnsInventoryCollection, NnsInventoryError,
        source::mainnet_registry_fetch_request,
    },
    report::{CollectionSourceRequest, SourceFuture},
};

///
/// NnsRegistryInventorySource
///
/// Collect all Registry inventory views at one exact version, without Governance metadata.
///

pub trait NnsRegistryInventorySource: Sync {
    /// Collect and project one complete pinned Registry relation graph.
    fn fetch_registry_inventory<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> SourceFuture<'a, Result<NnsRegistryInventorySnapshot, NnsInventoryError>>;
}

impl NnsRegistryInventorySource for LiveNnsSource {
    fn fetch_registry_inventory<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> SourceFuture<'a, Result<NnsRegistryInventorySnapshot, NnsInventoryError>> {
        Box::pin(async move {
            let request = mainnet_registry_fetch_request(request, |network| {
                NnsInventoryError::UnsupportedNetwork {
                    collection: NnsInventoryCollection::RegistryInventory,
                    network,
                }
            })?;
            crate::ic_registry::fetch_mainnet_registry_inventory(&request)
                .await
                .map_err(|source| NnsInventoryError::Registry {
                    collection: NnsInventoryCollection::RegistryInventory,
                    source,
                })
        })
    }
}
