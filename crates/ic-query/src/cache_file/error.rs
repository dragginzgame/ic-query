//! Module: cache_file::error
//!
//! Responsibility: typed errors for shared cache-file operations.
//! Does not own: command-specific error mapping or cache report schemas.
//! Boundary: names filesystem, atomic-write, and refresh-lock failures.

use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// CacheFileError
///
/// Generic file and refresh-lock failure returned by shared cache helpers.
///

#[derive(Debug, ThisError)]
pub enum CacheFileError {
    /// Creating the parent directory for a cache failed.
    #[error("failed to create cache directory at {}: {source}", path.display())]
    CreateDirectory {
        /// Directory path that could not be created.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Exclusively creating a refresh-lock file failed.
    #[error("failed to create refresh lock at {}: {source}", path.display())]
    CreateRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Reading an existing refresh-lock file failed.
    #[error("failed to read refresh lock at {}: {source}", path.display())]
    ReadRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// An existing refresh lock did not contain valid JSON.
    #[error(
        "failed to parse refresh lock at {}; remove the lock manually after verifying no refresh is running: {source}",
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
        "invalid refresh lock at {}; remove the lock manually after verifying no refresh is running: {reason}",
        path.display()
    )]
    InvalidRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Validation failure.
        reason: String,
    },

    /// Serializing a new refresh lock failed.
    #[error("failed to serialize refresh lock at {}: {source}", path.display())]
    SerializeRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },

    /// Writing a new refresh-lock file failed.
    #[error("failed to write refresh lock at {}: {source}", path.display())]
    WriteRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Removing a refresh-lock file after an operation failed.
    #[error("failed to remove refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Another refresh currently owns the cache lock.
    #[error("refresh already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        /// Refresh-lock path.
        path: PathBuf,
        /// Recorded lock acquisition time.
        started_at_unix_ms: u64,
    },

    /// An existing refresh lock is older than the caller's lock policy.
    #[error(
        "stale refresh lock exists at {} since unix_ms={started_at_unix_ms}; remove it manually after verifying no refresh is running",
        path.display()
    )]
    StaleRefreshLock {
        /// Refresh-lock path.
        path: PathBuf,
        /// Recorded lock acquisition time.
        started_at_unix_ms: u64,
    },

    /// Writing the temporary cache file failed.
    #[error("failed to write cache temp file at {}: {source}", path.display())]
    WriteTemp {
        /// Temporary cache path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Synchronizing the temporary cache file failed.
    #[error("failed to sync cache temp file at {}: {source}", path.display())]
    SyncTemp {
        /// Temporary cache path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Atomically replacing the final cache file failed.
    #[error("failed to replace cache at {} from {}: {source}", target_path.display(), temp_path.display())]
    Replace {
        /// Fully written temporary cache path.
        temp_path: PathBuf,
        /// Final cache path.
        target_path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Synchronizing the parent cache directory failed.
    #[error("failed to sync cache directory at {}: {source}", path.display())]
    SyncDirectory {
        /// Cache directory path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Writing a separately requested output file failed.
    #[error("failed to write cache output at {}: {source}", path.display())]
    WriteOutput {
        /// Output path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// Synchronizing a separately requested output file failed.
    #[error("failed to sync cache output at {}: {source}", path.display())]
    SyncOutput {
        /// Output path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },
}

///
/// HostCacheError
///
/// Component-labelled JSON cache and cache-operation failure shared by host reports.
///

#[derive(Debug, ThisError)]
pub enum HostCacheError {
    /// The requested component cache does not exist.
    #[error("{component} cache is missing at {}", path.display())]
    MissingCache {
        /// Component owning the cache.
        component: &'static str,
        /// Missing cache path.
        path: PathBuf,
    },

    /// Reading the requested component cache failed.
    #[error("failed to read {component} cache at {}: {source}", path.display())]
    ReadCache {
        /// Component owning the cache.
        component: &'static str,
        /// Cache path.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },

    /// The component cache did not contain valid JSON.
    #[error("failed to parse {component} cache at {}: {source}", path.display())]
    ParseCache {
        /// Component owning the cache.
        component: &'static str,
        /// Cache path.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },

    /// Serializing a component cache report failed.
    #[error("failed to serialize {component} cache JSON for {}: {source}", path.display())]
    SerializeCache {
        /// Component owning the cache.
        component: &'static str,
        /// Intended cache path.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },

    /// A component cache uses an unsupported schema version.
    #[error("unsupported {component} cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion {
        /// Component owning the cache.
        component: &'static str,
        /// Schema version found in the cache.
        version: u32,
        /// Schema version supported by the caller.
        expected: u32,
    },

    /// A component cache belongs to a different network namespace.
    #[error("cached {component} network mismatch: path is for {requested}, report is for {actual}")]
    NetworkMismatch {
        /// Component owning the cache.
        component: &'static str,
        /// Network requested by the caller.
        requested: String,
        /// Network recorded in the cache.
        actual: String,
    },

    /// A shared filesystem, lock, atomic-write, or output operation failed.
    #[error("{component} cache operation failed: {source}")]
    Operation {
        /// Component owning the cache operation.
        component: &'static str,
        /// Underlying shared cache-file failure.
        #[source]
        source: CacheFileError,
    },
}

impl HostCacheError {
    /// Build a typed missing-cache error for one component.
    #[must_use]
    pub const fn missing_cache(component: &'static str, path: PathBuf) -> Self {
        Self::MissingCache { component, path }
    }

    /// Build a typed cache-read error for one component.
    #[must_use]
    pub const fn read_cache(component: &'static str, path: PathBuf, source: io::Error) -> Self {
        Self::ReadCache {
            component,
            path,
            source,
        }
    }

    /// Build a typed cache-parse error for one component.
    #[must_use]
    pub const fn parse_cache(
        component: &'static str,
        path: PathBuf,
        source: serde_json::Error,
    ) -> Self {
        Self::ParseCache {
            component,
            path,
            source,
        }
    }

    /// Build a typed cache-serialization error for one component.
    #[must_use]
    pub const fn serialize_cache(
        component: &'static str,
        path: PathBuf,
        source: serde_json::Error,
    ) -> Self {
        Self::SerializeCache {
            component,
            path,
            source,
        }
    }

    /// Build a typed unsupported-schema error for one component.
    #[must_use]
    pub const fn unsupported_cache_schema_version(
        component: &'static str,
        version: u32,
        expected: u32,
    ) -> Self {
        Self::UnsupportedCacheSchemaVersion {
            component,
            version,
            expected,
        }
    }

    /// Build a typed cache-network mismatch for one component.
    #[must_use]
    pub const fn network_mismatch(
        component: &'static str,
        requested: String,
        actual: String,
    ) -> Self {
        Self::NetworkMismatch {
            component,
            requested,
            actual,
        }
    }

    /// Attach component context to a shared cache-file operation failure.
    #[must_use]
    pub const fn operation(component: &'static str, source: CacheFileError) -> Self {
        Self::Operation { component, source }
    }
}
