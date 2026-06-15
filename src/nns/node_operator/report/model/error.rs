use crate::ic_registry::RegistryFetchError;
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// NnsNodeOperatorHostError
///
#[derive(Debug, ThisError)]
pub enum NnsNodeOperatorHostError {
    #[error(
        "`icq nns node-operator` supports only the mainnet `ic` network\n\nThe NNS node-operator list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns node-operator list"
    )]
    UnsupportedNetwork { network: String },

    #[error("node-operator cache is missing at {}", path.display())]
    MissingCache { path: PathBuf },

    #[error("failed to read node-operator cache at {}: {source}", path.display())]
    ReadCache { path: PathBuf, source: io::Error },

    #[error("failed to parse node-operator cache at {}: {source}", path.display())]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize node-operator cache JSON for {}: {source}", path.display())]
    SerializeCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported node-operator cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion { version: u32, expected: u32 },

    #[error(
        "cached node-operator network mismatch: path is for {requested}, report is for {actual}"
    )]
    NetworkMismatch { requested: String, actual: String },

    #[error("node-operator refresh is already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        path: PathBuf,
        started_at_unix_ms: u64,
    },

    #[error("failed to create node-operator cache directory at {}: {source}", path.display())]
    CreateCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to create node-operator refresh lock at {}: {source}", path.display())]
    CreateRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to read node-operator refresh lock at {}: {source}", path.display())]
    ReadRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to parse node-operator refresh lock at {}: {source}", path.display())]
    ParseRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize node-operator refresh lock at {}: {source}", path.display())]
    SerializeRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to write node-operator refresh lock at {}: {source}", path.display())]
    WriteRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to remove node-operator refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock { path: PathBuf, source: io::Error },

    #[error("live NNS node-operator refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("failed to write node-operator cache temp file at {}: {source}", path.display())]
    WriteCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to sync node-operator cache temp file at {}: {source}", path.display())]
    SyncCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to replace node-operator cache at {} from {}: {source}", cache_path.display(), temp_path.display())]
    ReplaceCache {
        temp_path: PathBuf,
        cache_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync node-operator cache directory at {}: {source}", path.display())]
    SyncCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to write refreshed node-operator output at {}: {source}", path.display())]
    WriteRefreshOutput { path: PathBuf, source: io::Error },

    #[error("failed to sync refreshed node-operator output at {}: {source}", path.display())]
    SyncRefreshOutput { path: PathBuf, source: io::Error },

    #[error("node operator {input:?} did not match the mainnet NNS node-operator list")]
    NodeOperatorNotFound { input: String },

    #[error("node-operator prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodeOperatorPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}
