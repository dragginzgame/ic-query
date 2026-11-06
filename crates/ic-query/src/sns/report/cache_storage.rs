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
    },
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
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
/// SnsStoredCache
///
/// Complete stored SNS neuron snapshot.
///

pub(in crate::sns::report) type SnsStoredCache =
    SnapshotEnvelope<SnsCacheMetadata, super::neurons_cache::model::SnsNeuronsCacheRows>;

///
/// SnsCacheMetadata
///
/// Shared persisted identity metadata for a complete SNS collection cache.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::sns::report) struct SnsCacheMetadata {
    pub(in crate::sns::report) governance_canister_id: String,
}

/// Collect complete SNS snapshot paths for one cache collection.
pub(in crate::sns::report) fn collect_sns_cache_paths(
    cache_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError> {
    let root = sns_snapshot_network_cache_dir(cache_root, network);
    collect_full_collection_snapshot_paths(cache_root, &root, "neurons").map_err(|source| {
        SnsHostError::from(HostCacheError::operation(SNS_CACHE_COMPONENT, source))
    })
}

/// Load one complete SNS cache by root canister principal.
pub(in crate::sns::report) fn load_sns_cache_for_governance(
    cache_root: &Path,
    network: &str,
    governance_canister_id: &str,
) -> Result<(PathBuf, SnsStoredCache), SnsHostError> {
    enforce_mainnet_network(network)?;
    let governance_canister_id =
        super::target::parse_governance_canister_input(governance_canister_id)?;
    let path = sns_snapshot_cache_paths(cache_root, network, &governance_canister_id).snapshot_path;
    let cache = load_sns_cache_at(cache_root, path.clone(), network, None, None)?;
    Ok((path, cache))
}

/// Load and validate one complete SNS snapshot cache.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Consumes the selected path as part of snapshot admission and owned error projection."
)]
pub(in crate::sns::report) fn load_sns_cache_at(
    cache_root: &Path,
    path: PathBuf,
    network: &str,
    budget: Option<&mut ManagedReadBudget>,
    opened_file: Option<std::fs::File>,
) -> Result<SnsStoredCache, SnsHostError> {
    let key = sns_snapshot_key_for_cache_path(network, &path);
    let cache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            component: SNS_CACHE_COMPONENT,
            cache_root,
            path: path.clone(),
            network,
            expected_schema_version: super::neurons_cache::SNS_NEURONS_CACHE_SCHEMA_VERSION,
            maximum_bytes: crate::cache_file::MAX_COLLECTION_SNAPSHOT_BYTES,
        },
        budget,
        &key,
        |path| SnsHostError::MissingNeuronsCache { path },
        |completeness| SnsHostError::IncompleteRefresh {
            pages_fetched: completeness.page_count,
            rows_fetched: completeness.row_count,
            reason: "cached SNS neurons snapshot is not complete".to_string(),
        },
        |mismatch| sns_identity_mismatch_error(path.clone(), mismatch),
        opened_file,
    )?;
    validate_sns_cache(&path, &cache)?;
    Ok(cache)
}

fn validate_sns_cache(path: &Path, cache: &SnsStoredCache) -> Result<(), SnsHostError> {
    validate_cache_collection_completeness(&cache.completeness, cache.data.neurons.len())
        .map_err(|reason| invalid_sns_cache_error(path, reason))?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid_sns_cache_error(
            path,
            "SNS Governance neuron pagination cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    validate_sns_cache_metadata(path, &cache.metadata, &cache.entity)?;
    super::source::validate_sns_neuron_rows(&cache.data.neurons)
        .map_err(|reason| invalid_sns_cache_error(path, reason))
}

fn validate_sns_cache_metadata(
    path: &Path,
    metadata: &SnsCacheMetadata,
    entity: &str,
) -> Result<(), SnsHostError> {
    if metadata.governance_canister_id != entity {
        return Err(invalid_sns_cache_error(
            path,
            format!(
                "governance_canister_id is {}, expected {entity}",
                metadata.governance_canister_id
            ),
        ));
    }
    super::target::parse_governance_canister_input(&metadata.governance_canister_id)
        .map_err(|error| invalid_sns_cache_error(path, error.to_string()))?;
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
