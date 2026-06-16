mod cache_file;

use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum NnsLeafHostCacheError {
    #[error("{component} cache is missing at {}", path.display())]
    MissingCache {
        component: &'static str,
        path: PathBuf,
    },

    #[error("failed to read {component} cache at {}: {source}", path.display())]
    ReadCache {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to parse {component} cache at {}: {source}", path.display())]
    ParseCache {
        component: &'static str,
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize {component} cache JSON for {}: {source}", path.display())]
    SerializeCache {
        component: &'static str,
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported {component} cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion {
        component: &'static str,
        version: u32,
        expected: u32,
    },

    #[error("cached {component} network mismatch: path is for {requested}, report is for {actual}")]
    NetworkMismatch {
        component: &'static str,
        requested: String,
        actual: String,
    },

    #[error("{component} refresh is already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        component: &'static str,
        path: PathBuf,
        started_at_unix_ms: u64,
    },

    #[error("failed to create {component} cache directory at {}: {source}", path.display())]
    CreateCacheDirectory {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to create {component} refresh lock at {}: {source}", path.display())]
    CreateRefreshLock {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to read {component} refresh lock at {}: {source}", path.display())]
    ReadRefreshLock {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to parse {component} refresh lock at {}: {source}", path.display())]
    ParseRefreshLock {
        component: &'static str,
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize {component} refresh lock at {}: {source}", path.display())]
    SerializeRefreshLock {
        component: &'static str,
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to write {component} refresh lock at {}: {source}", path.display())]
    WriteRefreshLock {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to remove {component} refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to write {component} cache temp file at {}: {source}", path.display())]
    WriteCacheTemp {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync {component} cache temp file at {}: {source}", path.display())]
    SyncCacheTemp {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to replace {component} cache at {} from {}: {source}", cache_path.display(), temp_path.display())]
    ReplaceCache {
        component: &'static str,
        temp_path: PathBuf,
        cache_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync {component} cache directory at {}: {source}", path.display())]
    SyncCacheDirectory {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to write refreshed {component} output at {}: {source}", path.display())]
    WriteRefreshOutput {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync refreshed {component} output at {}: {source}", path.display())]
    SyncRefreshOutput {
        component: &'static str,
        path: PathBuf,
        source: io::Error,
    },
}

impl NnsLeafHostCacheError {
    pub const fn missing_cache(component: &'static str, path: PathBuf) -> Self {
        Self::MissingCache { component, path }
    }

    pub const fn read_cache(component: &'static str, path: PathBuf, source: io::Error) -> Self {
        Self::ReadCache {
            component,
            path,
            source,
        }
    }

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
}
