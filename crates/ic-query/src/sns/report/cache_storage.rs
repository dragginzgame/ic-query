//! Module: sns::report::cache_storage
//!
//! Responsibility: shared SNS snapshot storage and collection-family contract.
//! Does not own: cache payload models, refresh collection, or publication policy.
//! Boundary: centralizes discovery, direct root lookup, validation, loading, and error mapping.

use crate::{
    HostCacheError,
    cache::validate_cache_collection_completeness,
    cache_file::{CacheFileError, LoadJsonCacheRequest, ManagedReadBudget},
    snapshot_cache::{
        SnapshotEnvelope, SnapshotIdentityMismatch, collect_full_collection_snapshot_paths,
        load_complete_snapshot_for_key,
    },
    sns::report::{
        SNS_CACHE_COMPONENT, SnsHostError,
        cache_paths::{
            sns_snapshot_cache_paths, sns_snapshot_key_for_cache_path,
            sns_snapshot_network_cache_dir,
        },
        enforce_mainnet_network,
        lookup::parse_sns_root_canister_input,
    },
};
use serde::{Deserialize as SerdeDeserialize, Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

pub(in crate::sns::report) const SNS_CACHE_SCAN_MAXIMUM_BYTES: u64 = 1024 * 1024 * 1024;

pub(in crate::sns::report) const fn sns_cache_scan_limit_exceeded(error: &SnsHostError) -> bool {
    matches!(
        error,
        SnsHostError::Cache(HostCacheError::Operation {
            source: CacheFileError::ScanLimitExceeded { .. },
            ..
        })
    )
}

///
/// SnsCacheStorageFamily
///
/// Schema and missing-cache contract owned by one SNS snapshot collection.
///

pub(in crate::sns::report) trait SnsCacheStorageFamily {
    const COLLECTION: &'static str;

    type Data: DeserializeOwned + Serialize;

    const CACHE_SCHEMA_VERSION: u32;

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

///
/// SnsCacheMetadata
///
/// Shared persisted identity metadata for a complete SNS collection cache.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::sns::report) struct SnsCacheMetadata {
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
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

/// Load one complete SNS cache by root canister principal.
pub(in crate::sns::report) fn load_sns_cache_for_root<Family>(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> Result<SnsStoredCacheWithPath<Family>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    enforce_mainnet_network(network)?;
    let root_canister_id = parse_sns_root_canister_input(root_canister_id)?;
    let path =
        sns_snapshot_cache_paths::<Family>(cache_root, network, &root_canister_id).snapshot_path;
    let cache = load_sns_cache_at::<Family>(cache_root, path.clone(), network, None, None)?;
    Ok((path, cache))
}

/// Load and validate one complete SNS snapshot cache.
pub(in crate::sns::report) fn load_sns_cache_at<Family>(
    cache_root: &Path,
    path: PathBuf,
    network: &str,
    budget: Option<&mut ManagedReadBudget>,
    opened_file: Option<std::fs::File>,
) -> Result<SnsStoredCache<Family>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let key = sns_snapshot_key_for_cache_path::<Family>(network, &path);
    let cache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            component: SNS_CACHE_COMPONENT,
            cache_root,
            path: path.clone(),
            network,
            expected_schema_version: Family::CACHE_SCHEMA_VERSION,
            maximum_bytes: crate::cache_file::MAX_COLLECTION_SNAPSHOT_BYTES,
        },
        budget,
        &key,
        Family::missing_cache_error,
        |completeness| SnsHostError::IncompleteRefresh {
            pages_fetched: completeness.page_count,
            rows_fetched: completeness.row_count,
            reason: format!("cached SNS {} snapshot is not complete", Family::COLLECTION),
        },
        |mismatch| sns_identity_mismatch_error(path.clone(), mismatch),
        opened_file,
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

pub(in crate::sns::report) fn sns_identity_mismatch_error(
    path: PathBuf,
    mismatch: SnapshotIdentityMismatch,
) -> SnsHostError {
    SnsHostError::CacheIdentityMismatch {
        path,
        field: mismatch.field,
        expected: mismatch.expected,
        actual: mismatch.actual,
    }
}
