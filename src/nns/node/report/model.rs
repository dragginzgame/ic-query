use crate::{cache_file::JsonCacheReport, ic_registry::RegistryFetchError};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// NnsNodeCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

///
/// NnsNodeListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeListRequest {
    pub cache: NnsNodeCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub filters: NnsNodeListFilters,
}

///
/// NnsNodeInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeInfoRequest {
    pub cache: NnsNodeCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

///
/// NnsNodeRefreshRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeRefreshRequest {
    pub cache: NnsNodeCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

///
/// NnsNodeListFilters
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NnsNodeListFilters {
    pub subnet: Option<String>,
    pub subnet_kind: Option<String>,
    pub data_center: Option<String>,
    pub node_provider: Option<String>,
    pub node_operator: Option<String>,
}

impl NnsNodeListFilters {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.subnet.is_none()
            && self.subnet_kind.is_none()
            && self.data_center.is_none()
            && self.node_provider.is_none()
            && self.node_operator.is_none()
    }
}

///
/// NnsNodeListReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeListReport {
    pub schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_count: usize,
    pub nodes: Vec<NnsNodeRow>,
}

impl JsonCacheReport for NnsNodeListReport {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// NnsNodeRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeRow {
    pub node_principal: String,
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub subnet_principal: String,
    pub subnet_kind: String,
    pub data_center_id: String,
}

///
/// NnsNodeInfoReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_principal: String,
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub subnet_principal: String,
    pub subnet_kind: String,
    pub data_center_id: String,
}

///
/// NnsNodeRefreshReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_cache: bool,
    pub replaced_existing_cache: bool,
    pub node_count: usize,
}

///
/// NnsNodeHostError
///
#[derive(Debug, ThisError)]
pub enum NnsNodeHostError {
    #[error(
        "`icq nns node` supports only the mainnet `ic` network\n\nThe NNS node list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns node list"
    )]
    UnsupportedNetwork { network: String },

    #[error("node cache is missing at {}", path.display())]
    MissingCache { path: PathBuf },

    #[error("failed to read node cache at {}: {source}", path.display())]
    ReadCache { path: PathBuf, source: io::Error },

    #[error("failed to parse node cache at {}: {source}", path.display())]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize node cache JSON for {}: {source}", path.display())]
    SerializeCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported node cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion { version: u32, expected: u32 },

    #[error("cached node network mismatch: path is for {requested}, report is for {actual}")]
    NetworkMismatch { requested: String, actual: String },

    #[error("node refresh is already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        path: PathBuf,
        started_at_unix_ms: u64,
    },

    #[error("failed to create node cache directory at {}: {source}", path.display())]
    CreateCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to create node refresh lock at {}: {source}", path.display())]
    CreateRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to read node refresh lock at {}: {source}", path.display())]
    ReadRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to parse node refresh lock at {}: {source}", path.display())]
    ParseRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize node refresh lock at {}: {source}", path.display())]
    SerializeRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to write node refresh lock at {}: {source}", path.display())]
    WriteRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to remove node refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock { path: PathBuf, source: io::Error },

    #[error("live NNS node refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("failed to write node cache temp file at {}: {source}", path.display())]
    WriteCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to sync node cache temp file at {}: {source}", path.display())]
    SyncCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to replace node cache at {} from {}: {source}", cache_path.display(), temp_path.display())]
    ReplaceCache {
        temp_path: PathBuf,
        cache_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync node cache directory at {}: {source}", path.display())]
    SyncCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to write refreshed node output at {}: {source}", path.display())]
    WriteRefreshOutput { path: PathBuf, source: io::Error },

    #[error("failed to sync refreshed node output at {}: {source}", path.display())]
    SyncRefreshOutput { path: PathBuf, source: io::Error },

    #[error("node {input:?} did not match the mainnet NNS node list")]
    NodeNotFound { input: String },

    #[error("node prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodePrefix {
        prefix: String,
        matches: Vec<String>,
    },
}
