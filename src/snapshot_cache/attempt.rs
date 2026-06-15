use super::json::write_snapshot_json;
use crate::cache_file::CacheFileError;
use serde::{Deserialize as SerdeDeserialize, Serialize, de::DeserializeOwned};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnapshotRefreshAttempt<Metadata> {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub started_at: String,
    pub updated_at: String,
    #[serde(flatten)]
    pub metadata: Metadata,
    pub status: String,
    pub page_size: u32,
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
    pub last_error: Option<String>,
}

pub fn read_snapshot_refresh_attempt<T>(path: &Path) -> Option<T>
where
    T: DeserializeOwned,
{
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
}

pub fn write_snapshot_refresh_attempt<T, Error>(
    path: &Path,
    attempt: &T,
    serialize_error: impl FnOnce(PathBuf, serde_json::Error) -> Error,
    write_error: impl FnOnce(CacheFileError) -> Error,
) -> Result<(), Error>
where
    T: Serialize,
{
    write_snapshot_json(path, attempt, serialize_error, write_error)
}
