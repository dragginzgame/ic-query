//! Module: snapshot_cache::attempt
//!
//! Responsibility: read and write refresh-attempt sidecar files.
//! Does not own: command-specific attempt metadata or refresh execution.
//! Boundary: persists generic refresh-attempt JSON through cache-file primitives.

use super::json::write_snapshot_json;
use crate::cache_file::CacheFileError;
use serde::{Deserialize as SerdeDeserialize, Serialize, de::DeserializeOwned};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION: u32 = 1;

///
/// SnapshotRefreshAttemptReadError
///
/// Strict refresh-attempt sidecar read or parse failure.
///

#[derive(Debug)]
pub enum SnapshotRefreshAttemptReadError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

///
/// SnapshotRefreshAttempt
///
/// Sidecar status for an in-progress or failed complete snapshot refresh.
///

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

pub fn read_snapshot_refresh_attempt_strict<T>(
    path: &Path,
) -> Result<Option<T>, SnapshotRefreshAttemptReadError>
where
    T: DeserializeOwned,
{
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(SnapshotRefreshAttemptReadError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    serde_json::from_slice(&data).map(Some).map_err(|source| {
        SnapshotRefreshAttemptReadError::Parse {
            path: path.to_path_buf(),
            source,
        }
    })
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

pub fn current_attempt_timestamp(fallback: &str) -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| fallback.to_string(),
        |duration| crate::subnet_catalog::format_utc_timestamp_secs(duration.as_secs()),
    )
}
