//! Module: sns::report::cache_storage
//!
//! Responsibility: shared SNS snapshot cache storage helpers.
//! Does not own: family-specific schemas, error mappers, cache models, or refresh policy.
//! Boundary: builds common snapshot-cache load and discovery requests for SNS caches.

use crate::{
    cache_file::{LoadJsonCacheErrorMapper, LoadJsonCacheRequest},
    snapshot_cache::{
        SnapshotCompleteness, SnapshotHeader, SnapshotIdentityMismatch, SnapshotKey,
        SnapshotReport, collect_full_collection_snapshot_paths, load_complete_snapshot_for_key,
        load_snapshot_header,
    },
    sns::report::{
        SnsHostError,
        cache_paths::{SnsCacheCollection, sns_snapshot_network_cache_dir},
    },
};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

/// Collect complete SNS snapshot paths for one cache collection.
pub(in crate::sns::report) fn collect_sns_cache_paths<Collection>(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError>
where
    Collection: SnsCacheCollection,
{
    let root = sns_snapshot_network_cache_dir(icp_root, network);
    collect_full_collection_snapshot_paths(&root, Collection::COLLECTION)
        .map_err(|source| SnsHostError::ReadCache { path: root, source })
}

/// Read and validate one SNS snapshot cache header.
pub(in crate::sns::report) fn read_sns_cache_header<Metadata, Errors>(
    path: &Path,
    network: &str,
    expected_schema_version: u32,
    errors: Errors,
) -> Result<SnapshotHeader<Metadata>, SnsHostError>
where
    Metadata: DeserializeOwned,
    Errors: LoadJsonCacheErrorMapper<Error = SnsHostError>,
{
    load_snapshot_header(
        LoadJsonCacheRequest {
            path: path.to_path_buf(),
            network,
            expected_schema_version,
        },
        errors,
    )
}

/// Find the unique SNS snapshot path whose validated header claims an id.
pub(in crate::sns::report) fn find_unique_sns_cache_path_by_id(
    paths: impl IntoIterator<Item = PathBuf>,
    id: usize,
    mut read_id: impl FnMut(&Path) -> Result<usize, SnsHostError>,
) -> Result<Option<PathBuf>, SnsHostError> {
    let mut matching = None;
    for path in paths {
        if read_id(&path)? != id {
            continue;
        }
        if matching.replace(path).is_some() {
            return Err(SnsHostError::AmbiguousCacheId { id });
        }
    }
    Ok(matching)
}

/// Load and validate one complete SNS snapshot cache.
pub(in crate::sns::report) fn load_sns_complete_cache<Cache, Errors>(
    path: PathBuf,
    network: &str,
    expected_schema_version: u32,
    expected_key: &SnapshotKey,
    errors: Errors,
    incomplete_error: impl FnOnce(&SnapshotCompleteness) -> SnsHostError,
) -> Result<Cache, SnsHostError>
where
    Cache: DeserializeOwned + SnapshotReport,
    Errors: LoadJsonCacheErrorMapper<Error = SnsHostError>,
{
    load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            path: path.clone(),
            network,
            expected_schema_version,
        },
        expected_key,
        errors,
        incomplete_error,
        |mismatch| sns_identity_mismatch_error(path, mismatch),
    )
}

fn sns_identity_mismatch_error(path: PathBuf, mismatch: SnapshotIdentityMismatch) -> SnsHostError {
    SnsHostError::CacheIdentityMismatch {
        path,
        field: mismatch.field,
        expected: mismatch.expected,
        actual: mismatch.actual,
    }
}

#[cfg(test)]
mod tests {
    use super::find_unique_sns_cache_path_by_id;
    use crate::sns::report::SnsHostError;
    use std::path::PathBuf;

    #[test]
    fn cache_id_path_lookup_finds_the_unique_matching_header() {
        let path = find_unique_sns_cache_path_by_id(
            [PathBuf::from("1"), PathBuf::from("2"), PathBuf::from("3")],
            2,
            |path| {
                path.to_string_lossy()
                    .parse::<usize>()
                    .map_err(|_| SnsHostError::InvalidLookup {
                        input: path.display().to_string(),
                    })
            },
        )
        .expect("lookup succeeds");

        assert_eq!(path, Some(PathBuf::from("2")));
    }

    #[test]
    fn cache_id_path_lookup_rejects_duplicate_headers() {
        let result =
            find_unique_sns_cache_path_by_id([PathBuf::from("a"), PathBuf::from("b")], 7, |_| {
                Ok(7)
            });

        assert!(matches!(
            result,
            Err(SnsHostError::AmbiguousCacheId { id: 7 })
        ));
    }
}
