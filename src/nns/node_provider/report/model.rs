use crate::{cache_file::JsonCacheReport, ic_registry::RegistryFetchError};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// NnsNodeProviderCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

///
/// NnsNodeProviderListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderListRequest {
    pub cache: NnsNodeProviderCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsNodeProviderInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderInfoRequest {
    pub cache: NnsNodeProviderCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

///
/// NnsNodeProviderRefreshRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderRefreshRequest {
    pub cache: NnsNodeProviderCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

///
/// CachedNnsNodeProviderReport
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedNnsNodeProviderReport {
    pub path: PathBuf,
    pub report: NnsNodeProviderListReport,
}

///
/// NnsNodeProviderListReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderListReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_provider_count: usize,
    pub node_providers: Vec<NnsNodeProviderRow>,
}

impl JsonCacheReport for NnsNodeProviderListReport {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// NnsNodeProviderRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderRow {
    pub node_provider_principal: String,
    pub name: Option<String>,
    pub node_count: Option<u32>,
    pub reward_account_hex: Option<String>,
}

///
/// NnsNodeProviderInfoReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    pub network: String,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_provider_principal: String,
    pub name: Option<String>,
    pub node_count: Option<u32>,
    pub reward_account_hex: Option<String>,
}

///
/// NnsNodeProviderRefreshReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_cache: bool,
    pub replaced_existing_cache: bool,
    pub node_provider_count: usize,
}

///
/// NnsNodeProviderHostError
///
#[derive(Debug, ThisError)]
pub enum NnsNodeProviderHostError {
    #[error(
        "`icq nns node-provider` supports only the mainnet `ic` network\n\nThe NNS node-provider list is queried from the public Internet Computer mainnet governance canister.\nLocal replica NNS governance discovery is not implemented yet.\n\nTry:\n  icq --network ic nns node-provider list"
    )]
    UnsupportedNetwork { network: String },

    #[error("node-provider cache is missing at {}", path.display())]
    MissingCache { path: PathBuf },

    #[error("failed to read node-provider cache at {}: {source}", path.display())]
    ReadCache { path: PathBuf, source: io::Error },

    #[error("failed to parse node-provider cache at {}: {source}", path.display())]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize node-provider cache JSON for {}: {source}", path.display())]
    SerializeCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported node-provider cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion { version: u32, expected: u32 },

    #[error(
        "cached node-provider network mismatch: path is for {requested}, report is for {actual}"
    )]
    NetworkMismatch { requested: String, actual: String },

    #[error("node-provider refresh is already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        path: PathBuf,
        started_at_unix_ms: u64,
    },

    #[error("failed to create node-provider cache directory at {}: {source}", path.display())]
    CreateCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to create node-provider refresh lock at {}: {source}", path.display())]
    CreateRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to read node-provider refresh lock at {}: {source}", path.display())]
    ReadRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to parse node-provider refresh lock at {}: {source}", path.display())]
    ParseRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to write node-provider refresh lock at {}: {source}", path.display())]
    WriteRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to remove node-provider refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock { path: PathBuf, source: io::Error },

    #[error("live NNS node-provider refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("failed to write node-provider cache temp file at {}: {source}", path.display())]
    WriteCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to sync node-provider cache temp file at {}: {source}", path.display())]
    SyncCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to replace node-provider cache at {} from {}: {source}", cache_path.display(), temp_path.display())]
    ReplaceCache {
        temp_path: PathBuf,
        cache_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync node-provider cache directory at {}: {source}", path.display())]
    SyncCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to write refreshed node-provider output at {}: {source}", path.display())]
    WriteRefreshOutput { path: PathBuf, source: io::Error },

    #[error("failed to sync refreshed node-provider output at {}: {source}", path.display())]
    SyncRefreshOutput { path: PathBuf, source: io::Error },

    #[error("node provider {input:?} did not match the mainnet NNS node-provider list")]
    NodeProviderNotFound { input: String },

    #[error("node-provider prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodeProviderPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}
