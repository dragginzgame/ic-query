//! Module: snapshot_cache::json
//!
//! Responsibility: load and write shared complete-snapshot JSON files.
//! Does not own: snapshot path discovery, refresh attempts, or family-specific schemas.
//! Boundary: validates complete snapshot envelopes through cache-file JSON helpers.

use super::{SnapshotCompleteness, SnapshotHeader, SnapshotReport};
use crate::cache_file::{
    CacheFileError, CachedJsonReport, LoadJsonCacheErrorMapper, LoadJsonCacheRequest,
    load_json_cache, write_text_atomically,
};
use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

pub fn load_complete_snapshot<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    errors: Errors,
    incomplete_error: impl FnOnce(&SnapshotCompleteness) -> Errors::Error,
) -> Result<T, Errors::Error>
where
    T: DeserializeOwned + SnapshotReport,
    Errors: LoadJsonCacheErrorMapper,
{
    let cached: CachedJsonReport<T> = load_json_cache(request, errors)?;
    if !cached.report.completeness().is_api_exhausted() {
        return Err(incomplete_error(cached.report.completeness()));
    }
    Ok(cached.report)
}

pub fn load_snapshot_header<Metadata, Errors>(
    request: LoadJsonCacheRequest<'_>,
    errors: Errors,
) -> Result<SnapshotHeader<Metadata>, Errors::Error>
where
    Metadata: DeserializeOwned,
    Errors: LoadJsonCacheErrorMapper,
{
    let cached: CachedJsonReport<SnapshotHeader<Metadata>> = load_json_cache(request, errors)?;
    Ok(cached.report)
}

pub fn write_snapshot_json<T, Error>(
    path: &Path,
    snapshot: &T,
    serialize_error: impl FnOnce(PathBuf, serde_json::Error) -> Error,
    write_error: impl FnOnce(CacheFileError) -> Error,
) -> Result<(), Error>
where
    T: Serialize,
{
    let data = serde_json::to_string_pretty(snapshot)
        .map_err(|source| serialize_error(path.to_path_buf(), source))?;
    write_text_atomically(path, &data).map_err(write_error)
}
