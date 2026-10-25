//! Module: snapshot_cache::json
//!
//! Responsibility: load shared complete-snapshot JSON files.
//! Does not own: snapshot path discovery, refresh attempts, or family-specific schemas.
//! Boundary: validates complete snapshot envelopes through cache-file JSON helpers.

use super::{SnapshotEnvelope, SnapshotIdentityMismatch, SnapshotKey};
use crate::{
    cache::CacheCollectionCompleteness,
    cache_file::{
        CachedJsonReport, HostCacheError, LoadJsonCacheRequest, ManagedReadBudget, load_json_cache,
    },
};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub fn load_complete_snapshot_for_key<Metadata, Data, Error>(
    request: LoadJsonCacheRequest<'_>,
    budget: Option<&mut ManagedReadBudget>,
    key: &SnapshotKey,
    missing_error: impl FnOnce(PathBuf) -> Error,
    incomplete_error: impl FnOnce(&CacheCollectionCompleteness) -> Error,
    identity_error: impl FnOnce(SnapshotIdentityMismatch) -> Error,
    opened_file: Option<std::fs::File>,
) -> Result<SnapshotEnvelope<Metadata, Data>, Error>
where
    Metadata: DeserializeOwned,
    Data: DeserializeOwned,
    Error: From<HostCacheError>,
{
    let cached: CachedJsonReport<SnapshotEnvelope<Metadata, Data>> =
        load_json_cache(request, budget, opened_file).map_err(|error| match error {
            HostCacheError::MissingCache { path, .. } => missing_error(path),
            error => error.into(),
        })?;
    let snapshot = cached.report;
    if !snapshot.completeness.is_api_exhausted() {
        return Err(incomplete_error(&snapshot.completeness));
    }
    if let Some(mismatch) = snapshot_identity_mismatch(&snapshot, key) {
        return Err(identity_error(mismatch));
    }
    Ok(snapshot)
}

fn snapshot_identity_mismatch<Metadata, Data>(
    snapshot: &SnapshotEnvelope<Metadata, Data>,
    key: &SnapshotKey,
) -> Option<SnapshotIdentityMismatch> {
    identity_field_mismatch("domain", key.domain(), &snapshot.domain)
        .or_else(|| identity_field_mismatch("entity", key.entity(), &snapshot.entity))
        .or_else(|| identity_field_mismatch("collection", key.collection(), &snapshot.collection))
        .or_else(|| {
            identity_field_mismatch("scope", SnapshotKey::scope_file_stem(), &snapshot.scope)
        })
}

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
