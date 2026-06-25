use crate::{
    cache_file::CacheFileError,
    ic_registry::RegistryFetchError,
    subnet_catalog::{CatalogError, MAINNET_NETWORK},
};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

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

    #[error(
        "failed to parse refresh lock at {}; remove the lock manually after verifying no refresh is running: {source}",
        path.display()
    )]
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

pub(super) fn enforce_mainnet_network(network: &str) -> Result<(), SubnetCatalogHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(SubnetCatalogHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

pub(super) fn subnet_cache_error(err: CacheFileError) -> SubnetCatalogHostError {
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
