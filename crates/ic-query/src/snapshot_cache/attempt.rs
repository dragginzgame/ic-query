//! Module: snapshot_cache::attempt
//!
//! Responsibility: read and write refresh-attempt sidecar files.
//! Does not own: command-specific attempt metadata or refresh execution.
//! Boundary: persists generic refresh-attempt JSON through cache-file primitives.

use super::json::write_snapshot_json;
use crate::{
    cache::CacheRefreshAttemptStatus,
    cache_file::{CacheFileError, read_managed_file},
};
use serde::{Deserialize as SerdeDeserialize, Serialize, de::DeserializeOwned};
use std::{
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
    /// Capability-rooted cache access failed.
    Operation(CacheFileError),
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    Invalid {
        path: PathBuf,
        reason: String,
    },
}

///
/// SnapshotRefreshProgress
///
/// Page, row, and cursor progress shared by complete-snapshot refresh attempts.
///

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SnapshotRefreshProgress {
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
}

impl SnapshotRefreshProgress {
    #[must_use]
    pub const fn new(pages_fetched: u32, rows_fetched: usize, last_cursor: Option<String>) -> Self {
        Self {
            pages_fetched,
            rows_fetched,
            last_cursor,
        }
    }
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

pub fn read_snapshot_refresh_attempt_strict<T>(
    cache_root: &Path,
    path: &Path,
    metadata_fields: &[&str],
) -> Result<Option<T>, SnapshotRefreshAttemptReadError>
where
    T: DeserializeOwned,
{
    let Some(data) =
        read_managed_file(cache_root, path).map_err(SnapshotRefreshAttemptReadError::Operation)?
    else {
        return Ok(None);
    };
    let value: serde_json::Value =
        serde_json::from_slice(&data).map_err(|source| SnapshotRefreshAttemptReadError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
    if let Some(object) = value.as_object()
        && let Some(field) = object
            .keys()
            .find(|field| !attempt_field_is_supported(field, metadata_fields))
    {
        return Err(SnapshotRefreshAttemptReadError::Invalid {
            path: path.to_path_buf(),
            reason: format!("unknown field {field}"),
        });
    }
    serde_json::from_value(value).map(Some).map_err(|source| {
        SnapshotRefreshAttemptReadError::Parse {
            path: path.to_path_buf(),
            source,
        }
    })
}

fn attempt_field_is_supported(field: &str, metadata_fields: &[&str]) -> bool {
    matches!(
        field,
        "schema_version"
            | "network"
            | "source_endpoint"
            | "started_at"
            | "updated_at"
            | "status"
            | "page_size"
            | "pages_fetched"
            | "rows_fetched"
            | "last_cursor"
            | "last_error"
    ) || metadata_fields.contains(&field)
}

pub fn write_snapshot_refresh_attempt<T, Error>(
    cache_root: &Path,
    path: &Path,
    attempt: &T,
    serialize_error: impl FnOnce(PathBuf, serde_json::Error) -> Error,
    write_error: impl FnOnce(CacheFileError) -> Error,
) -> Result<(), Error>
where
    T: Serialize,
{
    write_snapshot_json(cache_root, path, attempt, serialize_error, write_error)
}

pub fn current_attempt_timestamp(fallback: &str) -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| fallback.to_string(),
        |duration| crate::subnet_catalog::format_utc_timestamp_secs(duration.as_secs()),
    )
}

pub fn validate_snapshot_refresh_attempt<Metadata>(
    attempt: &SnapshotRefreshAttempt<Metadata>,
    expected_network: &str,
) -> Result<CacheRefreshAttemptStatus, String> {
    if attempt.schema_version != SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION {
        return Err(format!(
            "schema_version is {}, expected {}",
            attempt.schema_version, SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION
        ));
    }
    if attempt.network != expected_network {
        return Err(format!(
            "network is {}, expected {expected_network}",
            attempt.network
        ));
    }
    if attempt.source_endpoint.is_empty() {
        return Err("source_endpoint must not be empty".to_string());
    }
    if attempt.started_at.is_empty() || attempt.updated_at.is_empty() {
        return Err("attempt timestamps must not be empty".to_string());
    }
    if attempt.page_size == 0 {
        return Err("page_size must be greater than zero".to_string());
    }
    if attempt.pages_fetched == 0 && (attempt.rows_fetched != 0 || attempt.last_cursor.is_some()) {
        return Err("zero-page attempt contains row or cursor progress".to_string());
    }
    let status = CacheRefreshAttemptStatus::from_label(&attempt.status)
        .ok_or_else(|| format!("unsupported attempt status {}", attempt.status))?;
    match status {
        CacheRefreshAttemptStatus::Running | CacheRefreshAttemptStatus::Complete
            if attempt.last_error.is_none() =>
        {
            Ok(status)
        }
        CacheRefreshAttemptStatus::Failed
            if attempt
                .last_error
                .as_deref()
                .is_some_and(|error| !error.trim().is_empty()) =>
        {
            Ok(status)
        }
        CacheRefreshAttemptStatus::Running | CacheRefreshAttemptStatus::Complete => {
            Err(format!("{status} attempt contains last_error"))
        }
        CacheRefreshAttemptStatus::Failed => {
            Err("failed attempt must contain last_error".to_string())
        }
    }
}
