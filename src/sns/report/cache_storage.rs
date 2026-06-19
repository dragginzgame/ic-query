//! Module: sns::report::cache_storage
//!
//! Responsibility: shared SNS snapshot cache storage helpers.
//! Does not own: family-specific schemas, error mappers, cache models, or refresh policy.
//! Boundary: builds common snapshot-cache load and discovery requests for SNS caches.

use crate::{
    cache_file::{LoadJsonCacheErrorMapper, LoadJsonCacheRequest},
    snapshot_cache::{
        SnapshotCompleteness, SnapshotHeader, SnapshotReport,
        collect_full_collection_snapshot_paths, load_complete_snapshot, load_snapshot_header,
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

/// Load and validate one complete SNS snapshot cache.
pub(in crate::sns::report) fn load_sns_complete_cache<Cache, Errors>(
    path: PathBuf,
    network: &str,
    expected_schema_version: u32,
    errors: Errors,
    incomplete_error: impl FnOnce(&SnapshotCompleteness) -> SnsHostError,
) -> Result<Cache, SnsHostError>
where
    Cache: DeserializeOwned + SnapshotReport,
    Errors: LoadJsonCacheErrorMapper<Error = SnsHostError>,
{
    load_complete_snapshot(
        LoadJsonCacheRequest {
            path,
            network,
            expected_schema_version,
        },
        errors,
        incomplete_error,
    )
}
