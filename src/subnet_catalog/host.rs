use super::{
    CatalogError, DEFAULT_REFRESH_LOCK_STALE_SECONDS, MAINNET_NETWORK,
    SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION, SubnetCatalog, SubnetCatalogRefreshReport,
    catalog_to_pretty_json, format_utc_timestamp_secs, parse_catalog_json,
};
use crate::{
    cache_file::{
        CacheFileError, RefreshLockRequest, announce_cache_refresh, create_parent_directory,
        with_refresh_lock, write_text_atomically, write_text_output,
    },
    ic_registry::{MainnetRegistryFetchRequest, RegistryFetchError, fetch_mainnet_subnet_catalog},
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error as ThisError;

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

///
/// SubnetCatalogRefreshRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogRefreshRequest {
    pub cache: SubnetCatalogCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

///
/// SubnetCatalogHostError
///
#[derive(Debug, ThisError)]
pub enum SubnetCatalogHostError {
    #[error(
        "`icq nns subnet` supports only the mainnet `ic` network\n\nThe cached NNS subnet data describes the public Internet Computer mainnet.\nLocal replica subnet discovery is not implemented yet.\n\nTry:\n  icq --network ic nns subnet list"
    )]
    UnsupportedNetwork { network: String },

    #[error(
        "subnet catalog cache is missing at {}\n\nRun `icq nns subnet refresh` to fetch the public Internet Computer mainnet catalog, or populate this path with a valid subnet catalog JSON.",
        path.display()
    )]
    MissingCatalog { path: PathBuf },

    #[error("failed to read subnet catalog at {}: {source}", path.display())]
    ReadCatalog { path: PathBuf, source: io::Error },

    #[error(
        "cached subnet catalog network mismatch: path is for {requested}, catalog is for {actual}"
    )]
    NetworkMismatch { requested: String, actual: String },

    #[error(
        "invalid stale duration {value:?}; use positive seconds or a value ending in s, m, h, or d"
    )]
    #[cfg(test)]
    InvalidStaleDuration { value: String },

    #[error("subnet catalog refresh is already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        path: PathBuf,
        started_at_unix_ms: u64,
    },

    #[error("failed to create subnet catalog directory at {}: {source}", path.display())]
    CreateCatalogDirectory { path: PathBuf, source: io::Error },

    #[error("failed to create refresh lock at {}: {source}", path.display())]
    CreateRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to read refresh lock at {}: {source}", path.display())]
    ReadRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to parse refresh lock at {}: {source}", path.display())]
    ParseRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize refresh lock at {}: {source}", path.display())]
    SerializeRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to write refresh lock at {}: {source}", path.display())]
    WriteRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to remove refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock { path: PathBuf, source: io::Error },

    #[error("live NNS registry refresh failed: {0}")]
    RegistryRefresh(#[from] RegistryFetchError),

    #[error("refreshed subnet catalog network mismatch: requested {requested}, fetched {actual}")]
    RefreshNetworkMismatch { requested: String, actual: String },

    #[error("failed to write subnet catalog temp file at {}: {source}", path.display())]
    WriteCatalogTemp { path: PathBuf, source: io::Error },

    #[error("failed to sync subnet catalog temp file at {}: {source}", path.display())]
    SyncCatalogTemp { path: PathBuf, source: io::Error },

    #[error("failed to replace subnet catalog at {} from {}: {source}", catalog_path.display(), temp_path.display())]
    ReplaceCatalog {
        temp_path: PathBuf,
        catalog_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync subnet catalog directory at {}: {source}", path.display())]
    SyncCatalogDirectory { path: PathBuf, source: io::Error },

    #[error("failed to write refreshed subnet catalog output at {}: {source}", path.display())]
    WriteRefreshOutput { path: PathBuf, source: io::Error },

    #[error("failed to sync refreshed subnet catalog output at {}: {source}", path.display())]
    SyncRefreshOutput { path: PathBuf, source: io::Error },

    #[error(transparent)]
    Catalog(#[from] CatalogError),
}

#[must_use]
pub fn subnet_catalog_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("subnet-catalog")
        .join(network)
        .join("catalog.json")
}

#[must_use]
pub fn subnet_catalog_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("subnet-catalog")
        .join(network)
        .join("refresh.lock")
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

pub fn refresh_subnet_catalog(
    request: &SubnetCatalogRefreshRequest,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    refresh_subnet_catalog_with_source(request, &LiveNnsRegistryRefreshSource)
}

pub fn refresh_subnet_catalog_with_source(
    request: &SubnetCatalogRefreshRequest,
    source: &dyn SubnetCatalogRefreshSource,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let catalog_path = subnet_catalog_path(&request.cache.icp_root, &request.cache.network);
    let lock_path =
        subnet_catalog_refresh_lock_path(&request.cache.icp_root, &request.cache.network);
    create_parent_directory(&catalog_path).map_err(subnet_cache_error)?;
    with_refresh_lock(
        RefreshLockRequest {
            lock_path: &lock_path,
            target_path: &catalog_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        subnet_cache_error,
        || {
            let replaced_existing_catalog = catalog_path.is_file();
            let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
            let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
            fetch_request.endpoint.clone_from(&request.source_endpoint);
            let catalog = source.fetch_catalog(&fetch_request)?;
            if catalog.network != request.cache.network {
                return Err(SubnetCatalogHostError::RefreshNetworkMismatch {
                    requested: request.cache.network.clone(),
                    actual: catalog.network,
                });
            }
            catalog.validate()?;
            let catalog_json = catalog_to_pretty_json(&catalog)?;
            if let Some(output_path) = &request.output_path {
                write_text_output(output_path, &catalog_json).map_err(subnet_cache_error)?;
            }
            if !request.dry_run {
                write_text_atomically(&catalog_path, &catalog_json).map_err(subnet_cache_error)?;
            }
            Ok(SubnetCatalogRefreshReport {
                schema_version: SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION,
                network: catalog.network,
                catalog_path: catalog_path.display().to_string(),
                refresh_lock_path: lock_path.display().to_string(),
                output_path: request
                    .output_path
                    .as_ref()
                    .map(|path| path.display().to_string()),
                registry_canister_id: catalog.registry_canister_id,
                registry_version: catalog.registry_version,
                fetched_at: catalog.fetched_at,
                source_endpoint: catalog.source_endpoint,
                fetched_by: catalog.fetched_by,
                dry_run: request.dry_run,
                wrote_catalog: !request.dry_run,
                replaced_existing_catalog,
                subnet_count: catalog.subnets.len(),
                routing_range_count: catalog.routing_ranges.len(),
            })
        },
    )
}

pub fn load_or_refresh_subnet_catalog(
    request: &SubnetCatalogCacheRequest,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn SubnetCatalogRefreshSource,
) -> Result<CachedSubnetCatalog, SubnetCatalogHostError> {
    match load_cached_subnet_catalog(request) {
        Ok(cached) => Ok(cached),
        Err(SubnetCatalogHostError::MissingCatalog { path }) => {
            announce_cache_refresh("subnet catalog", &path, source_endpoint);
            let refresh_request = SubnetCatalogRefreshRequest {
                cache: request.clone(),
                source_endpoint: source_endpoint.to_string(),
                now_unix_secs,
                lock_stale_after_seconds: DEFAULT_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            refresh_subnet_catalog_with_source(&refresh_request, source)?;
            load_cached_subnet_catalog(request)
        }
        Err(err) => Err(err),
    }
}

fn subnet_cache_error(err: CacheFileError) -> SubnetCatalogHostError {
    match err {
        CacheFileError::CreateDirectory { path, source } => {
            SubnetCatalogHostError::CreateCatalogDirectory { path, source }
        }
        CacheFileError::CreateRefreshLock { path, source } => {
            SubnetCatalogHostError::CreateRefreshLock { path, source }
        }
        CacheFileError::ReadRefreshLock { path, source } => {
            SubnetCatalogHostError::ReadRefreshLock { path, source }
        }
        CacheFileError::ParseRefreshLock { path, source } => {
            SubnetCatalogHostError::ParseRefreshLock { path, source }
        }
        CacheFileError::SerializeRefreshLock { path, source } => {
            SubnetCatalogHostError::SerializeRefreshLock { path, source }
        }
        CacheFileError::WriteRefreshLock { path, source } => {
            SubnetCatalogHostError::WriteRefreshLock { path, source }
        }
        CacheFileError::RemoveRefreshLock { path, source } => {
            SubnetCatalogHostError::RemoveRefreshLock { path, source }
        }
        CacheFileError::RefreshAlreadyInProgress {
            path,
            started_at_unix_ms,
        } => SubnetCatalogHostError::RefreshAlreadyInProgress {
            path,
            started_at_unix_ms,
        },
        CacheFileError::WriteTemp { path, source } => {
            SubnetCatalogHostError::WriteCatalogTemp { path, source }
        }
        CacheFileError::SyncTemp { path, source } => {
            SubnetCatalogHostError::SyncCatalogTemp { path, source }
        }
        CacheFileError::Replace {
            temp_path,
            target_path,
            source,
        } => SubnetCatalogHostError::ReplaceCatalog {
            temp_path,
            catalog_path: target_path,
            source,
        },
        CacheFileError::SyncDirectory { path, source } => {
            SubnetCatalogHostError::SyncCatalogDirectory { path, source }
        }
        CacheFileError::WriteOutput { path, source } => {
            SubnetCatalogHostError::WriteRefreshOutput { path, source }
        }
        CacheFileError::SyncOutput { path, source } => {
            SubnetCatalogHostError::SyncRefreshOutput { path, source }
        }
    }
}

fn enforce_mainnet_network(network: &str) -> Result<(), SubnetCatalogHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(SubnetCatalogHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

pub trait SubnetCatalogRefreshSource {
    fn fetch_catalog(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError>;
}

///
/// LiveNnsRegistryRefreshSource
///
pub struct LiveNnsRegistryRefreshSource;

impl SubnetCatalogRefreshSource for LiveNnsRegistryRefreshSource {
    fn fetch_catalog(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError> {
        Ok(fetch_mainnet_subnet_catalog(request)?)
    }
}
