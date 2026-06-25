//! Module: snapshot_cache::json
//!
//! Responsibility: load and write shared complete-snapshot JSON files.
//! Does not own: snapshot path discovery, refresh attempts, or family-specific schemas.
//! Boundary: validates complete snapshot envelopes through cache-file JSON helpers.

use super::{
    SnapshotCompleteness, SnapshotHeader, SnapshotIdentityMismatch, SnapshotKey, SnapshotReport,
};
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

pub fn load_complete_snapshot_for_key<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    key: &SnapshotKey,
    errors: Errors,
    incomplete_error: impl FnOnce(&SnapshotCompleteness) -> Errors::Error,
    identity_error: impl FnOnce(SnapshotIdentityMismatch) -> Errors::Error,
) -> Result<T, Errors::Error>
where
    T: DeserializeOwned + SnapshotReport,
    Errors: LoadJsonCacheErrorMapper,
{
    let snapshot = load_complete_snapshot(request, errors, incomplete_error)?;
    if let Some(mismatch) = snapshot_identity_mismatch(&snapshot, key) {
        return Err(identity_error(mismatch));
    }
    Ok(snapshot)
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

fn snapshot_identity_mismatch(
    snapshot: &impl SnapshotReport,
    key: &SnapshotKey,
) -> Option<SnapshotIdentityMismatch> {
    identity_field_mismatch("domain", key.domain(), snapshot.snapshot_domain())
        .or_else(|| identity_field_mismatch("entity", key.entity(), snapshot.snapshot_entity()))
        .or_else(|| {
            identity_field_mismatch(
                "collection",
                key.collection(),
                snapshot.snapshot_collection(),
            )
        })
        .or_else(|| {
            identity_field_mismatch("scope", key.scope_file_stem(), snapshot.snapshot_scope())
        })
}

fn identity_field_mismatch(
    field: &'static str,
    expected: &str,
    actual: Option<&str>,
) -> Option<SnapshotIdentityMismatch> {
    let actual = actual?;
    (actual != expected).then(|| SnapshotIdentityMismatch {
        field,
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
}
