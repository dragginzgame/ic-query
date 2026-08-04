use super::{
    SubnetCatalogHostError, SubnetCatalogRefreshRequest, SubnetCatalogSource,
    error::enforce_mainnet_network, refresh_subnet_catalog_with_source, subnet_catalog_path,
};
use crate::{
    cache_file::{CacheRefreshReason, load_or_refresh_cache_with_error_policy},
    nns::LiveNnsSource,
    subnet_catalog::{DEFAULT_REFRESH_LOCK_STALE_SECONDS, SubnetCatalog, parse_catalog_json},
};
use std::{fs, path::PathBuf};

///
/// SubnetCatalogCacheRequest
///
/// Cache root and network identity used to load a subnet catalog snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogCacheRequest {
    pub cache_root: PathBuf,
    pub network: String,
}

impl SubnetCatalogCacheRequest {
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }
}

///
/// CachedSubnetCatalog
///
/// Subnet catalog loaded from the host cache, including the path that supplied it.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedSubnetCatalog {
    pub path: PathBuf,
    pub catalog: SubnetCatalog,
}

/// Load a subnet catalog from the host cache without making live network calls.
pub fn load_cached_subnet_catalog(
    request: &SubnetCatalogCacheRequest,
) -> Result<CachedSubnetCatalog, SubnetCatalogHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = subnet_catalog_path(&request.cache_root, &request.network);
    if !path.is_file() {
        return Err(SubnetCatalogHostError::MissingCatalog { path });
    }
    let data = fs::read_to_string(&path).map_err(|source| SubnetCatalogHostError::ReadCatalog {
        path: path.clone(),
        source,
    })?;
    let catalog = parse_catalog_json(&data)?;
    if catalog.network != request.network {
        return Err(SubnetCatalogHostError::NetworkMismatch {
            requested: request.network.clone(),
            actual: catalog.network,
        });
    }
    Ok(CachedSubnetCatalog { path, catalog })
}

/// Load a subnet catalog from the host cache, refreshing recoverable local content failures.
pub fn load_or_refresh_subnet_catalog(
    request: &SubnetCatalogCacheRequest,
    source_endpoint: &str,
    now_unix_secs: u64,
) -> Result<CachedSubnetCatalog, SubnetCatalogHostError> {
    load_or_refresh_subnet_catalog_with_source(
        request,
        source_endpoint,
        now_unix_secs,
        &LiveNnsSource,
    )
}

pub fn load_or_refresh_subnet_catalog_with_source(
    request: &SubnetCatalogCacheRequest,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn SubnetCatalogSource,
) -> Result<CachedSubnetCatalog, SubnetCatalogHostError> {
    let expected_path = subnet_catalog_path(&request.cache_root, &request.network);
    load_or_refresh_cache_with_error_policy(
        || load_cached_subnet_catalog(request),
        |error| match error {
            SubnetCatalogHostError::MissingCatalog { path } => {
                Ok(CacheRefreshReason::Missing(path))
            }
            SubnetCatalogHostError::NetworkMismatch { .. } | SubnetCatalogHostError::Catalog(_) => {
                Ok(CacheRefreshReason::Invalid(expected_path.clone()))
            }
            error => Err(error),
        },
        |_| {
            let refresh_request = SubnetCatalogRefreshRequest::new(
                request.clone(),
                source_endpoint,
                now_unix_secs,
                DEFAULT_REFRESH_LOCK_STALE_SECONDS,
            );
            refresh_subnet_catalog_with_source(&refresh_request, source).map(|_| ())
        },
    )
}
