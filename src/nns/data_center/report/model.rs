use crate::{cache_file::JsonCacheReport, ic_registry::RegistryFetchError};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// NnsDataCenterCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

///
/// NnsDataCenterListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterListRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsDataCenterInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterInfoRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

///
/// NnsDataCenterRefreshRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterRefreshRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

///
/// CachedNnsDataCenterReport
///
#[derive(Clone, Debug, PartialEq)]
pub struct CachedNnsDataCenterReport {
    pub path: PathBuf,
    pub report: NnsDataCenterListReport,
}

///
/// NnsDataCenterListReport
///
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NnsDataCenterListReport {
    pub schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub data_center_count: usize,
    pub data_centers: Vec<NnsDataCenterRow>,
}

impl JsonCacheReport for NnsDataCenterListReport {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// NnsDataCenterRow
///
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NnsDataCenterRow {
    pub data_center_id: String,
    pub region: String,
    pub owner: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub node_operator_count: u32,
    pub node_provider_count: u32,
    pub node_count: u32,
}

///
/// NnsDataCenterInfoReport
///
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NnsDataCenterInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub data_center_id: String,
    pub region: String,
    pub owner: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub node_operator_count: u32,
    pub node_provider_count: u32,
    pub node_count: u32,
}

///
/// NnsDataCenterRefreshReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsDataCenterRefreshReport {
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
    pub data_center_count: usize,
}

///
/// NnsDataCenterHostError
///
#[derive(Debug, ThisError)]
pub enum NnsDataCenterHostError {
    #[error(
        "`icq nns data-center` supports only the mainnet `ic` network\n\nThe NNS data-center list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns data-center list"
    )]
    UnsupportedNetwork { network: String },

    #[error("data-center cache is missing at {}", path.display())]
    MissingCache { path: PathBuf },

    #[error("failed to read data-center cache at {}: {source}", path.display())]
    ReadCache { path: PathBuf, source: io::Error },

    #[error("failed to parse data-center cache at {}: {source}", path.display())]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize data-center cache JSON for {}: {source}", path.display())]
    SerializeCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported data-center cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion { version: u32, expected: u32 },

    #[error("cached data-center network mismatch: path is for {requested}, report is for {actual}")]
    NetworkMismatch { requested: String, actual: String },

    #[error("data-center refresh is already in progress; lock exists at {} since unix_ms={started_at_unix_ms}", path.display())]
    RefreshAlreadyInProgress {
        path: PathBuf,
        started_at_unix_ms: u64,
    },

    #[error("failed to create data-center cache directory at {}: {source}", path.display())]
    CreateCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to create data-center refresh lock at {}: {source}", path.display())]
    CreateRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to read data-center refresh lock at {}: {source}", path.display())]
    ReadRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to parse data-center refresh lock at {}: {source}", path.display())]
    ParseRefreshLock {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to write data-center refresh lock at {}: {source}", path.display())]
    WriteRefreshLock { path: PathBuf, source: io::Error },

    #[error("failed to remove data-center refresh lock at {}: {source}", path.display())]
    RemoveRefreshLock { path: PathBuf, source: io::Error },

    #[error("live NNS data-center refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("failed to write data-center cache temp file at {}: {source}", path.display())]
    WriteCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to sync data-center cache temp file at {}: {source}", path.display())]
    SyncCacheTemp { path: PathBuf, source: io::Error },

    #[error("failed to replace data-center cache at {} from {}: {source}", cache_path.display(), temp_path.display())]
    ReplaceCache {
        temp_path: PathBuf,
        cache_path: PathBuf,
        source: io::Error,
    },

    #[error("failed to sync data-center cache directory at {}: {source}", path.display())]
    SyncCacheDirectory { path: PathBuf, source: io::Error },

    #[error("failed to write refreshed data-center output at {}: {source}", path.display())]
    WriteRefreshOutput { path: PathBuf, source: io::Error },

    #[error("failed to sync refreshed data-center output at {}: {source}", path.display())]
    SyncRefreshOutput { path: PathBuf, source: io::Error },

    #[error("data center {input:?} did not match the mainnet NNS data-center list")]
    DataCenterNotFound { input: String },

    #[error("data-center prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousDataCenterPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}
