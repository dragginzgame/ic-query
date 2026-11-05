//! Refresh the shared Registry graph and independent Governance metadata.

use super::{NnsInventoryError, NnsInventoryRefreshRequest, source::NnsNodeProviderSource};
use crate::nns::{
    LiveNnsSource, NnsInventoryRefreshReport,
    inventory::{
        collect_registry_inventory, nns_registry_inventory_cache_path,
        nns_registry_inventory_refresh_lock_path,
    },
};

///
/// NnsNodeProviderRefreshReport
///
/// Independent publication outcomes for the Registry inventory and Governance metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct NnsNodeProviderRefreshReport {
    /// Shared complete Registry acquisition and publication.
    pub inventory: NnsInventoryRefreshReport,
    /// Complete Governance metadata acquisition and publication.
    pub metadata: NnsInventoryRefreshReport,
}

/// Refresh Registry counts and Governance provider metadata with native sources.
pub async fn refresh_nns_node_provider_report(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsNodeProviderRefreshReport, NnsInventoryError> {
    refresh_nns_node_provider_report_with_source(request, &LiveNnsSource::default()).await
}

/// Refresh both provider inputs while retaining independent complete-cache publication.
pub async fn refresh_nns_node_provider_report_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderRefreshReport, NnsInventoryError> {
    super::enforce_mainnet_network(&request.cache.network)?;
    if let Some(output) = &request.output_path {
        let paths = [
            nns_registry_inventory_cache_path(&request.cache.cache_root, &request.cache.network),
            nns_registry_inventory_refresh_lock_path(
                &request.cache.cache_root,
                &request.cache.network,
            ),
            super::cache::nns_node_provider_cache_path(
                &request.cache.cache_root,
                &request.cache.network,
            ),
            super::cache::nns_node_provider_refresh_lock_path(
                &request.cache.cache_root,
                &request.cache.network,
            ),
        ];
        let protected = paths
            .iter()
            .map(std::path::PathBuf::as_path)
            .collect::<Vec<_>>();
        crate::cache_file::validate_output_path(output, &protected).map_err(|source| {
            NnsInventoryError::Cache {
                collection: super::COLLECTION,
                source: crate::HostCacheError::operation("node-provider", source),
            }
        })?;
    }
    let mut inventory_request = request.clone();
    inventory_request.output_path = None;
    let (snapshot, inventory) = collect_registry_inventory(&inventory_request, source).await?;
    let (_, metadata) = super::cache::collect_metadata(request, source, &snapshot).await?;
    Ok(NnsNodeProviderRefreshReport {
        inventory,
        metadata,
    })
}
