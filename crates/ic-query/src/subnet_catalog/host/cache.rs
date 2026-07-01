use super::{
    LiveNnsRegistryRefreshSource, SubnetCatalogHostError, SubnetCatalogRefreshRequest,
    SubnetCatalogSource, error::enforce_mainnet_network, refresh_subnet_catalog_with_source,
    subnet_catalog_path,
};
use crate::{
    cache_file::load_or_refresh_missing_cache,
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
    pub icp_root: PathBuf,
    pub network: String,
}

impl SubnetCatalogCacheRequest {
    #[must_use]
    pub fn new(icp_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            icp_root: icp_root.into(),
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
    let path = subnet_catalog_path(&request.icp_root, &request.network);
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

/// Load a subnet catalog from the host cache, refreshing it from mainnet if it is missing.
pub fn load_or_refresh_subnet_catalog(
    request: &SubnetCatalogCacheRequest,
    source_endpoint: &str,
    now_unix_secs: u64,
) -> Result<CachedSubnetCatalog, SubnetCatalogHostError> {
    load_or_refresh_subnet_catalog_with_source(
        request,
        source_endpoint,
        now_unix_secs,
        &LiveNnsRegistryRefreshSource,
    )
}

pub fn load_or_refresh_subnet_catalog_with_source(
    request: &SubnetCatalogCacheRequest,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn SubnetCatalogSource,
) -> Result<CachedSubnetCatalog, SubnetCatalogHostError> {
    load_or_refresh_missing_cache(
        "subnet catalog",
        source_endpoint,
        || load_cached_subnet_catalog(request),
        || {
            let refresh_request = SubnetCatalogRefreshRequest::new(
                request.clone(),
                source_endpoint,
                now_unix_secs,
                DEFAULT_REFRESH_LOCK_STALE_SECONDS,
            );
            refresh_subnet_catalog_with_source(&refresh_request, source).map(|_| ())
        },
        |err| match err {
            SubnetCatalogHostError::MissingCatalog { path } => Ok(path),
            err => Err(err),
        },
    )
}
