//! Module: snapshot_cache::json
//!
//! Responsibility: load and write shared complete-snapshot JSON files.
//! Does not own: snapshot path discovery, refresh attempts, or family-specific schemas.
//! Boundary: validates complete snapshot envelopes through cache-file JSON helpers.

#[cfg(feature = "sns-host")]
use super::SnapshotHeader;
#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
use super::{SnapshotIdentityMismatch, SnapshotKey, SnapshotReport};
use crate::cache_file::{CacheFileError, write_managed_json_pretty_atomically};
#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
use crate::{
    cache::CacheCollectionCompleteness,
    cache_file::{
        CachedJsonReport, LoadJsonCacheErrorMapper, LoadJsonCacheRequest, load_json_cache_strict,
    },
};
use serde::Serialize;
#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
pub fn load_complete_snapshot<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    supported_fields: &'static [&'static str],
    errors: Errors,
    incomplete_error: impl FnOnce(&CacheCollectionCompleteness) -> Errors::Error,
) -> Result<T, Errors::Error>
where
    T: DeserializeOwned + SnapshotReport,
    Errors: LoadJsonCacheErrorMapper,
{
    let cached: CachedJsonReport<T> = load_json_cache_strict(request, supported_fields, errors)?;
    if !cached.report.completeness().is_api_exhausted() {
        return Err(incomplete_error(cached.report.completeness()));
    }
    Ok(cached.report)
}

#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
pub fn load_complete_snapshot_for_key<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    key: &SnapshotKey,
    supported_fields: &'static [&'static str],
    errors: Errors,
    incomplete_error: impl FnOnce(&CacheCollectionCompleteness) -> Errors::Error,
    identity_error: impl FnOnce(SnapshotIdentityMismatch) -> Errors::Error,
) -> Result<T, Errors::Error>
where
    T: DeserializeOwned + SnapshotReport,
    Errors: LoadJsonCacheErrorMapper,
{
    let snapshot = load_complete_snapshot(request, supported_fields, errors, incomplete_error)?;
    if let Some(mismatch) = snapshot_identity_mismatch(&snapshot, key) {
        return Err(identity_error(mismatch));
    }
    Ok(snapshot)
}

#[cfg(feature = "sns-host")]
pub fn load_snapshot_header<Metadata, Errors>(
    request: LoadJsonCacheRequest<'_>,
    supported_fields: &'static [&'static str],
    errors: Errors,
) -> Result<SnapshotHeader<Metadata>, Errors::Error>
where
    Metadata: DeserializeOwned,
    Errors: LoadJsonCacheErrorMapper,
{
    let cached: CachedJsonReport<SnapshotHeader<Metadata>> =
        load_json_cache_strict(request, supported_fields, errors)?;
    Ok(cached.report)
}

pub fn write_snapshot_json<T, Error>(
    cache_root: &Path,
    path: &Path,
    snapshot: &T,
    serialize_error: impl FnOnce(PathBuf, serde_json::Error) -> Error,
    write_error: impl FnOnce(CacheFileError) -> Error,
) -> Result<(), Error>
where
    T: Serialize,
{
    write_managed_json_pretty_atomically(cache_root, path, snapshot, serialize_error, write_error)
}

#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
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

#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
fn identity_field_mismatch(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Option<SnapshotIdentityMismatch> {
    (actual != expected).then(|| SnapshotIdentityMismatch {
        field,
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
}
