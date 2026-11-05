//! Shared Registry snapshot lifecycle and NNS row resolution.

mod model;
mod source;
pub use model::NnsRegistryInventorySnapshot;
pub use source::NnsRegistryInventorySource;

use super::{
    LiveNnsSource, NnsInventoryCacheRequest, NnsInventoryCollection, NnsInventoryError,
    NnsInventoryListRequest, NnsInventoryRefreshRequest,
};
use crate::{
    QueryProgress, QueryProgressEvent,
    cache_file::{RefreshCacheWriteResult, load_or_refresh_cache},
    identifier_match::{IdentifierMatch, match_identifier},
    report::CollectionSourceRequest,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, canonical_principal_text},
};
use ic_host_text::timestamp::parse_utc_timestamp_secs;
use serde::Serialize;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

const COLLECTION: NnsInventoryCollection = NnsInventoryCollection::RegistryInventory;
const COMPONENT: &str = "inventory";
const CACHE_FILE: &str = "snapshot.json";

///
/// NnsInventoryRefreshReport
///
/// Publication outcome shared by Registry-derived NNS inventory refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, Serialize)]
pub struct NnsInventoryRefreshReport {
    /// Serialized report schema version, currently 1.
    pub schema_version: u32,
    /// Registry inventory represented by this refresh.
    pub collection: String,
    /// Number of rows collected before any view filtering.
    pub row_count: usize,
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    /// Managed complete inventory cache path.
    pub cache_path: String,
    /// Refresh lock path.
    pub refresh_lock_path: String,
    /// Explicit export path, when requested.
    pub output_path: Option<String>,
    /// Governance principal supplying provider metadata, when applicable.
    pub governance_canister_id: Option<String>,
    /// Independently collected Governance metadata, when applicable.
    pub governance_source: Option<CollectionSourceRequest>,
    /// Whether managed-cache publication was disabled.
    pub dry_run: bool,
    /// Whether the managed complete cache was published.
    pub wrote_cache: bool,
    /// Whether a cache existed at write admission, including during a dry run.
    pub replaced_existing_cache: bool,
}

/// Return the shared complete Registry snapshot path.
#[must_use]
pub fn nns_registry_inventory_cache_path(root: &Path, network: &str) -> PathBuf {
    root.join("nns")
        .join(network)
        .join(COMPONENT)
        .join(CACHE_FILE)
}

/// Return the shared Registry refresh exclusion path.
#[must_use]
pub fn nns_registry_inventory_refresh_lock_path(root: &Path, network: &str) -> PathBuf {
    root.join("nns")
        .join(network)
        .join(COMPONENT)
        .join("refresh.lock")
}

impl NnsRegistryInventorySnapshot {
    pub(in crate::nns) fn validate(&self) -> Result<(), String> {
        validate_inventory_metadata(
            self.schema_version,
            &self.provenance.network,
            &self.provenance.source_endpoint,
            &self.provenance.fetched_at,
        )?;
        if self.provenance.registry_canister_id != MAINNET_REGISTRY_CANISTER_ID {
            return Err(format!(
                "registry_canister_id is {}, expected {MAINNET_REGISTRY_CANISTER_ID}",
                self.provenance.registry_canister_id
            ));
        }
        Ok(())
    }
}

pub(in crate::nns) fn validate_inventory_metadata(
    schema: u32,
    network: &str,
    endpoint: &str,
    fetched_at: &str,
) -> Result<(), String> {
    if schema != 1 {
        return Err(format!("schema_version is {schema}, expected 1"));
    }
    if network != MAINNET_NETWORK {
        return Err(format!("network is {network}, expected {MAINNET_NETWORK}"));
    }
    if parse_utc_timestamp_secs(fetched_at).is_none() {
        return Err(format!("fetched_at is invalid: {fetched_at:?}"));
    }
    if endpoint.trim().is_empty() {
        return Err("source_endpoint must not be empty".to_string());
    }
    Ok(())
}

pub(in crate::nns) fn validate_inventory_source(
    request: &CollectionSourceRequest,
    network: &str,
    endpoint: &str,
    fetched_at: &str,
) -> Result<(), String> {
    for (field, actual, expected) in [
        ("network", network, request.network.as_str()),
        ("source_endpoint", endpoint, request.endpoint.as_str()),
        ("fetched_at", fetched_at, request.fetched_at.as_str()),
    ] {
        if actual != expected {
            return Err(format!("{field} is {actual:?}, requested {expected:?}"));
        }
    }
    Ok(())
}

pub(in crate::nns) fn load_registry_inventory(
    cache: &NnsInventoryCacheRequest,
) -> Result<NnsRegistryInventorySnapshot, NnsInventoryError> {
    crate::network::enforce_mainnet_network_with(&cache.network, |network| {
        NnsInventoryError::UnsupportedNetwork {
            collection: COLLECTION,
            network,
        }
    })?;
    let cached = super::leaf::load_nns_leaf_json_cache::<NnsRegistryInventorySnapshot>(
        cache, COMPONENT, CACHE_FILE, 1,
    )
    .map_err(|source| NnsInventoryError::Cache {
        collection: COLLECTION,
        source,
    })?;
    cached
        .report
        .validate()
        .map_err(|reason| NnsInventoryError::Cache {
            collection: COLLECTION,
            source: crate::HostCacheError::invalid_cache(COMPONENT, cached.path, reason),
        })?;
    Ok(cached.report)
}

pub(in crate::nns) async fn load_or_refresh_nns_inventory_report(
    request: &NnsInventoryListRequest,
    source: &dyn NnsRegistryInventorySource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsRegistryInventorySnapshot, NnsInventoryError> {
    let path = nns_registry_inventory_cache_path(&request.cache.cache_root, &request.cache.network);
    load_or_refresh_cache(
        || load_registry_inventory(&request.cache),
        |_| false,
        |error| error.cache_refresh_reason(path.clone()),
        async |_| {
            progress.report(QueryProgressEvent::CacheRefresh {
                component: COMPONENT.to_string(),
                path: path.clone(),
                source_endpoint: request.source_endpoint.clone(),
            });
            refresh_nns_registry_inventory_with_source(
                &NnsInventoryRefreshRequest::new(
                    request.cache.clone(),
                    &request.source_endpoint,
                    request.now_unix_secs,
                ),
                source,
            )
            .await
            .map(|_| ())
        },
    )
    .await
}

/// Refresh the shared Registry snapshot through the built-in native source.
pub async fn refresh_nns_registry_inventory(
    request: &NnsInventoryRefreshRequest,
) -> Result<NnsInventoryRefreshReport, NnsInventoryError> {
    refresh_nns_registry_inventory_with_source(request, &LiveNnsSource::default()).await
}

/// Refresh all Registry inventory views through one caller-owned source.
pub async fn refresh_nns_registry_inventory_with_source(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsRegistryInventorySource,
) -> Result<NnsInventoryRefreshReport, NnsInventoryError> {
    collect_registry_inventory(request, source)
        .await
        .map(|(_, report)| report)
}

pub(in crate::nns) async fn collect_registry_inventory(
    request: &NnsInventoryRefreshRequest,
    source: &dyn NnsRegistryInventorySource,
) -> Result<(NnsRegistryInventorySnapshot, NnsInventoryRefreshReport), NnsInventoryError> {
    crate::network::enforce_mainnet_network_with(&request.cache.network, |network| {
        NnsInventoryError::UnsupportedNetwork {
            collection: COLLECTION,
            network,
        }
    })?;
    let source_request = CollectionSourceRequest::from_unix_secs(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query",
    );
    let snapshot = source.fetch_registry_inventory(&source_request).await?;
    snapshot
        .validate()
        .and_then(|()| {
            validate_inventory_source(
                &source_request,
                &snapshot.provenance.network,
                &snapshot.provenance.source_endpoint,
                &snapshot.provenance.fetched_at,
            )
        })
        .map_err(|reason| NnsInventoryError::InvalidSourceData {
            collection: COLLECTION,
            reason,
        })?;
    let write =
        super::leaf::write_nns_leaf_json_refresh_cache(request, COMPONENT, CACHE_FILE, &snapshot)
            .await
            .map_err(|source| NnsInventoryError::Cache {
                collection: COLLECTION,
                source,
            })?;
    let report = inventory_refresh_outcome(
        request,
        COMPONENT,
        snapshot.nodes.len() + snapshot.node_operators.len() + snapshot.data_centers.len(),
        snapshot.provenance.clone(),
        None,
        write,
    );
    Ok((snapshot, report))
}

pub(in crate::nns) fn inventory_refresh_outcome(
    request: &NnsInventoryRefreshRequest,
    collection: &str,
    row_count: usize,
    provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    governance_source: Option<CollectionSourceRequest>,
    write: RefreshCacheWriteResult,
) -> NnsInventoryRefreshReport {
    NnsInventoryRefreshReport {
        schema_version: 1,
        collection: collection.to_string(),
        row_count,
        provenance,
        cache_path: write.cache_path,
        refresh_lock_path: write.refresh_lock_path,
        output_path: write.output_path,
        governance_canister_id: governance_source
            .as_ref()
            .map(|_| super::MAINNET_GOVERNANCE_CANISTER_ID.to_string()),
        governance_source,
        dry_run: request.dry_run,
        wrote_cache: write.wrote_cache,
        replaced_existing_cache: write.replaced_existing_cache,
    }
}

///
/// NnsInventoryInputKind
///
/// Identifier normalization used by one Registry inventory family.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::nns) enum NnsInventoryInputKind {
    /// Canonical IC principal with principal-prefix fallback.
    Principal,
    /// Trimmed, lowercase textual identifier.
    Text,
}

///
/// NnsInventoryResolveError
///
/// Family-independent failure from exact-or-unique-prefix row resolution.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) enum NnsInventoryResolveError {
    /// No row matched the original input.
    NotFound {
        /// Original caller input retained for the family error.
        input: String,
    },
    /// More than one row matched the normalized prefix.
    Ambiguous {
        /// Normalized lowercase prefix.
        prefix: String,
        /// Canonically ordered identifiers that matched.
        matches: Vec<String>,
    },
}

impl NnsInventoryResolveError {
    pub(super) fn for_collection(self, collection: NnsInventoryCollection) -> NnsInventoryError {
        match self {
            Self::NotFound { input } => NnsInventoryError::NotFound { collection, input },
            Self::Ambiguous { prefix, matches } => NnsInventoryError::AmbiguousPrefix {
                collection,
                prefix,
                matches,
            },
        }
    }
}

/// Resolve a Registry inventory row by exact normalized identifier or unique prefix.
pub(in crate::nns) fn resolve_nns_inventory_row<Row>(
    rows: &[Row],
    identifier: impl Fn(&Row) -> &str,
    input: &str,
    input_kind: NnsInventoryInputKind,
    exact_source: &'static str,
) -> Result<(usize, String), NnsInventoryResolveError> {
    let prefix = input.trim().to_ascii_lowercase();
    let exact = match input_kind {
        NnsInventoryInputKind::Principal => canonical_principal_text(input).ok().map(Cow::Owned),
        NnsInventoryInputKind::Text => {
            (!prefix.is_empty()).then_some(Cow::Borrowed(prefix.as_str()))
        }
    };
    match match_identifier(
        rows.iter().enumerate(),
        |(_, row)| identifier(row),
        exact.as_deref(),
        &prefix,
    ) {
        IdentifierMatch::Exact((index, _)) => Ok((index, exact_source.to_string())),
        IdentifierMatch::Prefix((index, _)) => Ok((index, format!("{exact_source}_prefix"))),
        IdentifierMatch::Missing => Err(NnsInventoryResolveError::NotFound {
            input: input.to_string(),
        }),
        IdentifierMatch::Ambiguous(matches) => Err(NnsInventoryResolveError::Ambiguous {
            prefix,
            matches: matches
                .into_iter()
                .map(|(_, row)| identifier(row).to_string())
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests;
