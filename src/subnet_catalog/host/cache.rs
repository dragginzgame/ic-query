use super::{
    SubnetCatalogHostError, SubnetCatalogRefreshRequest, SubnetCatalogRefreshSource,
    error::enforce_mainnet_network, refresh_subnet_catalog_with_source, subnet_catalog_path,
};
use crate::{
    cache_file::load_or_refresh_missing_cache,
    subnet_catalog::{DEFAULT_REFRESH_LOCK_STALE_SECONDS, SubnetCatalog, parse_catalog_json},
};
use std::{fs, path::PathBuf};

///
/// SubnetCatalogCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

///
/// CachedSubnetCatalog
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedSubnetCatalog {
    pub path: PathBuf,
    pub catalog: SubnetCatalog,
}

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

pub fn load_or_refresh_subnet_catalog(
    request: &SubnetCatalogCacheRequest,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn SubnetCatalogRefreshSource,
) -> Result<CachedSubnetCatalog, SubnetCatalogHostError> {
    load_or_refresh_missing_cache(
        "subnet catalog",
        source_endpoint,
        || load_cached_subnet_catalog(request),
        || {
            let refresh_request = SubnetCatalogRefreshRequest {
                cache: request.clone(),
                source_endpoint: source_endpoint.to_string(),
                now_unix_secs,
                lock_stale_after_seconds: DEFAULT_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            refresh_subnet_catalog_with_source(&refresh_request, source).map(|_| ())
        },
        |err| match err {
            SubnetCatalogHostError::MissingCatalog { path } => Ok(path),
            err => Err(err),
        },
    )
}
