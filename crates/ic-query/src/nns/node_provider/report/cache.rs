//! Governance-only provider metadata cache; Registry counts are joined at read time.

use super::{NnsNodeProviderMetadata, NnsNodeProviderSource};
use crate::{
    QueryProgress, QueryProgressEvent,
    cache_file::load_or_refresh_cache,
    nns::{
        NnsInventoryCacheRequest, NnsInventoryError, NnsInventoryListRequest,
        NnsInventoryRefreshRequest, NnsRegistryInventorySnapshot,
        inventory::{
            inventory_refresh_outcome, validate_inventory_metadata, validate_inventory_source,
        },
    },
    report::CollectionSourceRequest,
};
use std::path::{Path, PathBuf};

const COMPONENT: &str = "node-provider";
const CACHE_FILE: &str = "metadata.json";

/// Return the complete Governance provider metadata path.
#[must_use]
pub fn nns_node_provider_cache_path(root: &Path, network: &str) -> PathBuf {
    root.join("nns")
        .join(network)
        .join(COMPONENT)
        .join(CACHE_FILE)
}
/// Return the Governance provider metadata refresh lock path.
#[must_use]
pub fn nns_node_provider_refresh_lock_path(root: &Path, network: &str) -> PathBuf {
    root.join("nns")
        .join(network)
        .join(COMPONENT)
        .join("refresh.lock")
}

impl NnsNodeProviderMetadata {
    pub(in crate::nns) fn validate(&self) -> Result<(), String> {
        validate_inventory_metadata(
            self.schema_version,
            &self.source.network,
            &self.source.endpoint,
            &self.source.fetched_at,
        )?;
        if self.governance_canister_id != crate::nns::MAINNET_GOVERNANCE_CANISTER_ID {
            return Err(format!(
                "governance_canister_id is {}, expected {}",
                self.governance_canister_id,
                crate::nns::MAINNET_GOVERNANCE_CANISTER_ID
            ));
        }
        if self
            .node_providers
            .iter()
            .any(|row| row.node_count.is_some())
        {
            return Err("Governance metadata must not claim Registry node counts".to_string());
        }
        Ok(())
    }
}

fn load_metadata(
    cache: &NnsInventoryCacheRequest,
) -> Result<NnsNodeProviderMetadata, NnsInventoryError> {
    super::enforce_mainnet_network(&cache.network)?;
    let cached = crate::nns::leaf::load_nns_leaf_json_cache::<NnsNodeProviderMetadata>(
        cache, COMPONENT, CACHE_FILE, 1,
    )
    .map_err(|source| NnsInventoryError::Cache {
        collection: super::COLLECTION,
        source,
    })?;
    cached
        .report
        .validate()
        .map_err(|reason| NnsInventoryError::Cache {
            collection: super::COLLECTION,
            source: crate::HostCacheError::invalid_cache(COMPONENT, cached.path, reason),
        })?;
    Ok(cached.report)
}

pub(in crate::nns) async fn load_or_refresh_metadata(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeProviderSource,
    snapshot: &NnsRegistryInventorySnapshot,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeProviderMetadata, NnsInventoryError> {
    let path = nns_node_provider_cache_path(&request.cache.cache_root, &request.cache.network);
    load_or_refresh_cache(
        || load_metadata(&request.cache),
        |_| false,
        |error| error.cache_refresh_reason(path.clone()),
        async |_| {
            progress.report(QueryProgressEvent::CacheRefresh {
                component: "node-provider metadata".to_string(),
                path: path.clone(),
                source_endpoint: request.source_endpoint.clone(),
            });
            collect_metadata(
                &NnsInventoryRefreshRequest::new(
                    request.cache.clone(),
                    &request.source_endpoint,
                    request.now_unix_secs,
                ),
                source,
                snapshot,
            )
            .await
            .map(|_| ())
        },
    )
    .await
}

pub(super) async fn collect_metadata(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsNodeProviderSource,
    snapshot: &NnsRegistryInventorySnapshot,
) -> Result<
    (
        NnsNodeProviderMetadata,
        crate::nns::NnsInventoryRefreshReport,
    ),
    NnsInventoryError,
> {
    let expected = CollectionSourceRequest::from_unix_secs(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query",
    );
    let metadata = source.fetch_node_provider_metadata(&expected).await?;
    metadata
        .validate()
        .and_then(|()| {
            validate_inventory_source(
                &expected,
                &metadata.source.network,
                &metadata.source.endpoint,
                &metadata.source.fetched_at,
            )
        })
        .map_err(|reason| NnsInventoryError::InvalidSourceData {
            collection: super::COLLECTION,
            reason,
        })?;
    let write = crate::nns::leaf::write_nns_leaf_json_refresh_cache(
        request, COMPONENT, CACHE_FILE, &metadata,
    )
    .map_err(|source| NnsInventoryError::Cache {
        collection: super::COLLECTION,
        source,
    })?;
    let report = inventory_refresh_outcome(
        request,
        "node_provider_metadata",
        metadata.node_providers.len(),
        snapshot.provenance.clone(),
        Some(metadata.source.clone()),
        write,
    );
    Ok((metadata, report))
}
