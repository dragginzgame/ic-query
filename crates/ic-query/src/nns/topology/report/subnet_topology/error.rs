use crate::{
    cache_file::CacheFileError, ic_registry::RegistryFetchError,
    nns::topology::report::subnet_topology::NnsSubnetTopologyValidationError,
    subnet_catalog::MAINNET_NETWORK,
};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// NnsSubnetTopologyHostError
///
/// Live-source, relation-validation, cache, and refresh-lock failures.
///

#[derive(Debug, ThisError)]
pub enum NnsSubnetTopologyHostError {
    /// A caller requested a network other than mainnet.
    #[error("NNS Subnet topology supports only the mainnet `ic` network; requested {network}")]
    UnsupportedNetwork {
        /// Unsupported network name.
        network: String,
    },

    /// No joined topology cache exists at the canonical path.
    #[error("Subnet topology cache is missing at {}", path.display())]
    MissingCache {
        /// Missing cache path.
        path: PathBuf,
    },

    /// Reading the joined cache failed.
    #[error("failed to read Subnet topology cache at {}: {source}", path.display())]
    ReadCache {
        /// Cache path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// The joined cache did not contain valid report JSON.
    #[error("failed to parse Subnet topology cache at {}: {source}", path.display())]
    ParseCache {
        /// Cache path.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },

    /// Serializing a refreshed report failed.
    #[error("failed to serialize Subnet topology cache at {}: {source}", path.display())]
    SerializeCache {
        /// Intended cache path.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },

    /// A cached report uses an unsupported schema.
    #[error("unsupported Subnet topology cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion {
        /// Schema version found in the cache.
        version: u32,
        /// Schema version supported by this library.
        expected: u32,
    },

    /// The cached report belongs to a different network namespace.
    #[error(
        "cached Subnet topology network mismatch: path is for {requested}, report is for {actual}"
    )]
    CacheNetworkMismatch {
        /// Network requested by the caller.
        requested: String,
        /// Network recorded in the cached report.
        actual: String,
    },

    /// A source returned a report for a different network.
    #[error("refreshed Subnet topology network mismatch: requested {requested}, fetched {actual}")]
    RefreshNetworkMismatch {
        /// Network requested by the caller.
        requested: String,
        /// Network recorded in the refreshed report.
        actual: String,
    },

    /// A source returned data attributed to an unexpected Registry canister.
    #[error(
        "refreshed Subnet topology Registry canister mismatch: expected {expected}, fetched {actual}"
    )]
    RegistryCanisterMismatch {
        /// Expected mainnet Registry principal.
        expected: String,
        /// Registry principal recorded in the refreshed report.
        actual: String,
    },

    /// A source returned data attributed to a different endpoint.
    #[error(
        "refreshed Subnet topology source endpoint mismatch: requested {requested}, fetched {actual}"
    )]
    SourceEndpointMismatch {
        /// Endpoint requested by the caller.
        requested: String,
        /// Endpoint recorded in the refreshed report.
        actual: String,
    },

    /// Creating the joined cache directory failed.
    #[error("failed to create Subnet topology cache directory at {}: {source}", path.display())]
    CreateCacheDirectory {
        /// Cache directory path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Creating the exclusive refresh lock failed.
    #[error("failed to create Subnet topology refresh lock at {}: {source}", path.display())]
    CreateRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Reading an existing refresh lock failed.
    #[error("failed to read Subnet topology refresh lock at {}: {source}", path.display())]
    ReadRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// An existing refresh lock did not contain valid JSON.
    #[error(
        "failed to parse Subnet topology refresh lock at {}; remove it manually after verifying no refresh is running: {source}",
        path.display()
    )]
    ParseRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },

    /// An existing refresh lock failed semantic validation.
    #[error(
        "invalid Subnet topology refresh lock at {}; remove it manually after verifying no refresh is running: {reason}",
        path.display()
    )]
    InvalidRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Validation failure.
        reason: String,
    },

    /// Serializing a new refresh lock failed.
    #[error("failed to serialize Subnet topology refresh lock at {}: {source}", path.display())]
    SerializeRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },

    /// Writing the refresh lock failed.
    #[error("failed to write Subnet topology refresh lock at {}: {source}", path.display())]
    WriteRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Removing the refresh lock after collection failed.
    #[error("failed to remove Subnet topology refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Another refresh currently owns the joined cache lock.
    #[error(
        "Subnet topology refresh is already in progress; lock exists at {} since unix_ms={started_at_unix_ms}",
        path.display()
    )]
    RefreshAlreadyInProgress {
        /// Refresh-lock path.
        path: PathBuf,
        /// Recorded lock acquisition time.
        started_at_unix_ms: u64,
    },

    /// An existing refresh lock is older than the caller's lock policy.
    #[error(
        "stale Subnet topology refresh lock exists at {} since unix_ms={started_at_unix_ms}; remove it manually after verifying no refresh is running",
        path.display()
    )]
    StaleRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Recorded lock acquisition time.
        started_at_unix_ms: u64,
    },

    /// Writing the temporary cache file failed.
    #[error("failed to write Subnet topology cache temp file at {}: {source}", path.display())]
    WriteCacheTemp {
        /// Temporary cache path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Synchronizing the temporary cache file failed.
    #[error("failed to sync Subnet topology cache temp file at {}: {source}", path.display())]
    SyncCacheTemp {
        /// Temporary cache path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Atomically replacing the joined cache failed.
    #[error(
        "failed to replace Subnet topology cache at {} from {}: {source}",
        cache_path.display(),
        temp_path.display()
    )]
    ReplaceCache {
        /// Fully written temporary cache path.
        temp_path: PathBuf,
        /// Final joined cache path.
        cache_path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Synchronizing the cache directory after replacement failed.
    #[error("failed to sync Subnet topology cache directory at {}: {source}", path.display())]
    SyncCacheDirectory {
        /// Cache directory path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// A reusable cache helper unexpectedly attempted a separate output write.
    #[error("unexpected Subnet topology output write at {}: {source}", path.display())]
    WriteOutput {
        /// Unexpected output path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// A reusable cache helper unexpectedly attempted to sync separate output.
    #[error("unexpected Subnet topology output sync at {}: {source}", path.display())]
    SyncOutput {
        /// Unexpected output path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Exact-version Registry collection or relation projection failed.
    #[error(transparent)]
    Registry(#[from] RegistryFetchError),

    /// Refreshed or cached report invariants failed validation.
    #[error(transparent)]
    Validation(#[from] NnsSubnetTopologyValidationError),
}

impl From<CacheFileError> for NnsSubnetTopologyHostError {
    fn from(error: CacheFileError) -> Self {
        match error {
            CacheFileError::CreateDirectory { path, source } => {
                Self::CreateCacheDirectory { path, source }
            }
            CacheFileError::CreateRefreshLock { path, source } => {
                Self::CreateRefreshLock { path, source }
            }
            CacheFileError::ReadRefreshLock { path, source } => {
                Self::ReadRefreshLock { path, source }
            }
            CacheFileError::ParseRefreshLock { path, source } => {
                Self::ParseRefreshLock { path, source }
            }
            CacheFileError::InvalidRefreshLock { path, reason } => {
                Self::InvalidRefreshLock { path, reason }
            }
            CacheFileError::SerializeRefreshLock { path, source } => {
                Self::SerializeRefreshLock { path, source }
            }
            CacheFileError::WriteRefreshLock { path, source } => {
                Self::WriteRefreshLock { path, source }
            }
            CacheFileError::RemoveRefreshLock { path, source } => {
                Self::RemoveRefreshLock { path, source }
            }
            CacheFileError::RefreshAlreadyInProgress {
                path,
                started_at_unix_ms,
            } => Self::RefreshAlreadyInProgress {
                path,
                started_at_unix_ms,
            },
            CacheFileError::StaleRefreshLock {
                path,
                started_at_unix_ms,
            } => Self::StaleRefreshLock {
                path,
                started_at_unix_ms,
            },
            CacheFileError::WriteTemp { path, source } => Self::WriteCacheTemp { path, source },
            CacheFileError::SyncTemp { path, source } => Self::SyncCacheTemp { path, source },
            CacheFileError::Replace {
                temp_path,
                target_path,
                source,
            } => Self::ReplaceCache {
                temp_path,
                cache_path: target_path,
                source,
            },
            CacheFileError::SyncDirectory { path, source } => {
                Self::SyncCacheDirectory { path, source }
            }
            CacheFileError::WriteOutput { path, source } => Self::WriteOutput { path, source },
            CacheFileError::SyncOutput { path, source } => Self::SyncOutput { path, source },
        }
    }
}

pub(super) fn enforce_mainnet_network(network: &str) -> Result<(), NnsSubnetTopologyHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(NnsSubnetTopologyHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}
