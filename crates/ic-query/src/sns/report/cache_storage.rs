//! Module: sns::report::cache_storage
//!
//! Responsibility: shared SNS snapshot storage and collection-family contract.
//! Does not own: cache payload models, refresh collection, or publication policy.
//! Boundary: centralizes discovery, strict id/root lookup, validation, loading, and error mapping.

use crate::{
    HostCacheError,
    cache::{CacheCollectionCompleteness, validate_cache_collection_completeness},
    cache_file::{LoadJsonCacheRequest, OwnerJsonCacheErrorMapper},
    snapshot_cache::{
        SnapshotEnvelope, SnapshotHeader, SnapshotIdentityMismatch,
        collect_full_collection_snapshot_paths, load_complete_snapshot_for_key,
        load_snapshot_header,
    },
    sns::report::{
        SNS_CACHE_COMPONENT, SnsHostError,
        cache_paths::{
            SnsCacheCollection, SnsSnapshotCachePaths, sns_snapshot_key_for_cache_path,
            sns_snapshot_network_cache_dir,
        },
    },
};
use serde::{Deserialize as SerdeDeserialize, Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

///
/// SnsCacheStorageFamily
///
/// Schema and missing-cache contract owned by one SNS snapshot collection.
///

pub(in crate::sns::report) trait SnsCacheStorageFamily:
    SnsCacheCollection
{
    type Data: DeserializeOwned;

    const CACHE_SCHEMA_VERSION: u32;
    const CACHE_FIELDS: &'static [&'static str];
    const CACHE_ITEM_NAME: &'static str;

    fn missing_cache_error(path: PathBuf) -> SnsHostError;
    fn row_count(data: &Self::Data) -> usize;
    fn validate_rows(data: &Self::Data) -> Result<(), String>;
}

///
/// SnsStoredCache
///
/// Complete stored snapshot type associated with one SNS cache family.
///

pub(in crate::sns::report) type SnsStoredCache<Family> =
    SnapshotEnvelope<SnsCacheMetadata, <Family as SnsCacheStorageFamily>::Data>;

///
/// SnsStoredCacheWithPath
///
/// Loaded SNS snapshot paired with its concrete complete-cache path.
///

pub(in crate::sns::report) type SnsStoredCacheWithPath<Family> = (PathBuf, SnsStoredCache<Family>);

#[derive(Clone, Copy)]
struct SnsCacheLoadErrors {
    collection: &'static str,
    missing_cache_error: fn(PathBuf) -> SnsHostError,
}

impl SnsCacheLoadErrors {
    fn for_family<Family>() -> Self
    where
        Family: SnsCacheStorageFamily,
    {
        Self {
            collection: Family::COLLECTION,
            missing_cache_error: Family::missing_cache_error,
        }
    }

    fn incomplete_cache_error(self, completeness: &CacheCollectionCompleteness) -> SnsHostError {
        SnsHostError::IncompleteRefresh {
            pages_fetched: completeness.page_count,
            rows_fetched: completeness.row_count,
            reason: format!("cached SNS {} snapshot is not complete", self.collection),
        }
    }

    fn json_errors(self) -> OwnerJsonCacheErrorMapper<SnsHostError> {
        OwnerJsonCacheErrorMapper::new(SNS_CACHE_COMPONENT, self.missing_cache_error)
    }
}

///
/// SnsCacheMetadata
///
/// Shared persisted identity metadata for a complete SNS collection cache.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::sns::report) struct SnsCacheMetadata {
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
}

///
/// SnsCacheHeaderMetadata
///
/// Minimal SNS metadata loaded while scanning collection cache headers.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
pub(in crate::sns::report) struct SnsCacheHeaderMetadata {
    pub(in crate::sns::report) id: usize,
}

/// Collect complete SNS snapshot paths for one cache collection.
pub(in crate::sns::report) fn collect_sns_cache_paths<Family>(
    cache_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let root = sns_snapshot_network_cache_dir(cache_root, network);
    collect_full_collection_snapshot_paths(cache_root, &root, Family::COLLECTION).map_err(
        |source| SnsHostError::from(HostCacheError::operation(SNS_CACHE_COMPONENT, source)),
    )
}

/// Read and validate one SNS snapshot cache header.
pub(in crate::sns::report) fn read_sns_cache_header<Family>(
    cache_root: &Path,
    path: &Path,
    network: &str,
) -> Result<SnapshotHeader<SnsCacheHeaderMetadata>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    load_snapshot_header(
        LoadJsonCacheRequest {
            cache_root,
            path: path.to_path_buf(),
            network,
            expected_schema_version: Family::CACHE_SCHEMA_VERSION,
        },
        Family::CACHE_FIELDS,
        SnsCacheLoadErrors::for_family::<Family>().json_errors(),
    )
}

/// Find the unique SNS snapshot path whose validated header claims an id.
fn find_unique_sns_cache_path_by_id(
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

/// Load the unique complete SNS cache whose validated header claims an id.
pub(in crate::sns::report) fn load_sns_cache_by_id<Family>(
    cache_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<SnsStoredCacheWithPath<Family>>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let path = find_unique_sns_cache_path_by_id(
        collect_sns_cache_paths::<Family>(cache_root, network)?,
        id,
        |path| {
            read_sns_cache_header::<Family>(cache_root, path, network)
                .map(|header| header.metadata.id)
        },
    )?;
    path.map(|path| {
        load_sns_cache_at::<Family>(cache_root, path.clone(), network).map(|cache| (path, cache))
    })
    .transpose()
}

/// Load one complete SNS cache by root canister principal.
pub(in crate::sns::report) fn load_sns_cache_for_root<Family>(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> Result<SnsStoredCacheWithPath<Family>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let path =
        SnsSnapshotCachePaths::<Family>::for_root(cache_root, network, root_canister_id).cache_path;
    let cache = load_sns_cache_at::<Family>(cache_root, path.clone(), network)?;
    Ok((path, cache))
}

/// Load and validate one complete SNS snapshot cache.
pub(in crate::sns::report) fn load_sns_cache_at<Family>(
    cache_root: &Path,
    path: PathBuf,
    network: &str,
) -> Result<SnsStoredCache<Family>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let key = sns_snapshot_key_for_cache_path::<Family>(network, &path);
    let errors = SnsCacheLoadErrors::for_family::<Family>();
    let cache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            cache_root,
            path: path.clone(),
            network,
            expected_schema_version: Family::CACHE_SCHEMA_VERSION,
        },
        &key,
        Family::CACHE_FIELDS,
        errors.json_errors(),
        |completeness| errors.incomplete_cache_error(completeness),
        |mismatch| sns_identity_mismatch_error(path.clone(), mismatch),
    )?;
    validate_sns_cache::<Family>(&path, &cache)?;
    Ok(cache)
}

fn validate_sns_cache<Family>(
    path: &Path,
    cache: &SnsStoredCache<Family>,
) -> Result<(), SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    validate_cache_collection_completeness(&cache.completeness, Family::row_count(&cache.data))
        .map_err(|reason| invalid_sns_cache_error(path, reason))?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid_sns_cache_error(
            path,
            format!(
                "SNS Governance {} pagination cannot claim a point-in-time guarantee",
                Family::CACHE_ITEM_NAME
            ),
        ));
    }
    validate_sns_cache_metadata(path, &cache.metadata, &cache.entity)?;
    Family::validate_rows(&cache.data).map_err(|reason| invalid_sns_cache_error(path, reason))
}

fn validate_sns_cache_metadata(
    path: &Path,
    metadata: &SnsCacheMetadata,
    entity: &str,
) -> Result<(), SnsHostError> {
    if metadata.id == 0 {
        return Err(invalid_sns_cache_error(
            path,
            "SNS list id must be greater than zero".to_string(),
        ));
    }
    if metadata.root_canister_id != entity {
        return Err(invalid_sns_cache_error(
            path,
            format!(
                "root_canister_id is {}, expected {entity}",
                metadata.root_canister_id
            ),
        ));
    }
    if metadata.governance_canister_id.is_empty() {
        return Err(invalid_sns_cache_error(
            path,
            "governance_canister_id must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn invalid_sns_cache_error(path: &Path, reason: String) -> SnsHostError {
    SnsHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    }
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
