//! Shared status and summary projection for native Governance collection caches.

use super::{
    NnsGovernanceAttemptReadError, NnsGovernanceCacheMetadata, NnsGovernanceCacheRequest,
    NnsGovernanceRefreshAttemptStatus, read_governance_refresh_attempt_status,
};
use crate::{
    HostCacheError,
    cache::{
        CacheCollectionCompleteness, CacheValidationStatus, validate_cache_collection_completeness,
    },
    cache_file::open_managed_file,
    nns::MAINNET_GOVERNANCE_CANISTER_ID,
    snapshot_cache::{SnapshotEnvelope, SnapshotJsonPaths},
};
use serde::Serialize;
use std::{fs::File, path::Path};

pub(in crate::nns) fn build_governance_cache_status_report<Error>(
    request: &NnsGovernanceCacheRequest,
    collection: &str,
    component: &'static str,
    paths: &SnapshotJsonPaths,
    summary: impl FnOnce(File) -> NnsGovernanceCacheSummary,
) -> Result<NnsGovernanceCacheStatusReport, Error>
where
    Error: From<HostCacheError> + From<NnsGovernanceAttemptReadError>,
{
    let file = open_managed_file(&request.cache_root, &paths.snapshot_path)
        .map_err(|source| HostCacheError::operation(component, source))?;
    let cache = file.map(summary);
    let latest_attempt = read_governance_refresh_attempt_status(
        &request.cache_root,
        &paths.refresh_attempt_path,
        &request.network,
        component,
    )?;
    Ok(NnsGovernanceCacheStatusReport::from_cache(
        request,
        collection,
        paths,
        cache,
        latest_attempt,
    ))
}

///
/// NnsGovernanceCacheSummary
///
/// Validation and collection evidence for one complete or invalid Governance snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsGovernanceCacheSummary {
    /// Expected Governance authority, or the identity admitted from a valid snapshot.
    pub governance_canister_id: String,
    /// Snapshot validation outcome.
    pub cache_status: CacheValidationStatus,
    /// Diagnostic for an invalid snapshot.
    pub cache_error: Option<String>,
    /// Whether API exhaustion established completeness.
    pub complete: bool,
    /// Whether every row describes one Governance instant.
    pub point_in_time_guaranteed: bool,
    /// Admitted row count.
    pub row_count: usize,
    /// Admitted page count.
    pub page_count: u32,
    /// Requested rows per collection page.
    pub page_size: u32,
    /// Collection timestamp from valid evidence; empty for invalid content.
    pub fetched_at: String,
    /// Collection endpoint from valid evidence; empty for invalid content.
    pub source_endpoint: String,
    /// Path of the inspected complete snapshot.
    pub cache_path: String,
}

impl NnsGovernanceCacheSummary {
    /// Project a complete snapshot after its family loader admitted identity, rows and completeness.
    pub(in crate::nns) fn from_snapshot<Rows>(
        path: &Path,
        cache: &SnapshotEnvelope<NnsGovernanceCacheMetadata, Rows>,
    ) -> Self {
        Self {
            governance_canister_id: cache.metadata.governance_canister_id.clone(),
            cache_status: CacheValidationStatus::Valid,
            cache_error: None,
            complete: cache.completeness.is_api_exhausted(),
            point_in_time_guaranteed: cache.completeness.point_in_time_guaranteed,
            row_count: cache.completeness.row_count,
            page_count: cache.completeness.page_count,
            page_size: cache.completeness.page_size,
            fetched_at: cache.fetched_at.clone(),
            source_endpoint: cache.source_endpoint.clone(),
            cache_path: path.display().to_string(),
        }
    }

    /// Retain refusal evidence without projecting unvalidated snapshot values.
    pub(in crate::nns) fn invalid(path: &Path, reason: String) -> Self {
        Self {
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
            cache_status: CacheValidationStatus::Invalid,
            cache_error: Some(reason),
            complete: false,
            point_in_time_guaranteed: false,
            row_count: 0,
            page_count: 0,
            page_size: 0,
            fetched_at: String::new(),
            source_endpoint: String::new(),
            cache_path: path.display().to_string(),
        }
    }
}

///
/// NnsGovernanceCacheStatusReport
///
/// Local inspection of one Governance collection and its independent latest refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsGovernanceCacheStatusReport {
    /// Current report schema; always 1 before package 1.0.
    pub schema_version: u32,
    /// Requested cache network namespace.
    pub network: String,
    /// Inspected collection, neurons or proposals.
    pub collection: String,
    /// Directory containing the expected snapshot.
    pub cache_root: String,
    /// Whether the expected snapshot exists, including invalid content.
    pub found: bool,
    /// Snapshot validation summary when present.
    pub cache: Option<NnsGovernanceCacheSummary>,
    /// Expected complete snapshot path.
    pub expected_cache_path: String,
    /// Expected independent refresh-attempt path.
    pub refresh_attempt_path: String,
    /// Latest valid refresh-attempt evidence, independent of the last complete cache.
    pub latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
}

impl NnsGovernanceCacheStatusReport {
    /// Assemble status from family-owned loading and attempt admission.
    pub(in crate::nns) fn from_cache(
        request: &NnsGovernanceCacheRequest,
        collection: &str,
        paths: &SnapshotJsonPaths,
        cache: Option<NnsGovernanceCacheSummary>,
        latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
    ) -> Self {
        Self {
            schema_version: 1,
            network: request.network.clone(),
            collection: collection.to_string(),
            cache_root: paths
                .snapshot_path
                .parent()
                .unwrap_or(&request.cache_root)
                .display()
                .to_string(),
            found: cache.is_some(),
            cache,
            expected_cache_path: paths.snapshot_path.display().to_string(),
            refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
            latest_attempt,
        }
    }
}

// Both collection families persist the same snapshot identity and accounting contract.
macro_rules! validate_governance_snapshot {
    ($cache:expr, $state:expr, $rows:ident, $validate:path) => {{
        let cache = $cache;
        let state = $state;
        $validate(state)
            .map_err(|error| error.to_string())
            .and_then(|()| {
                let source = $crate::nns::governance::NnsGovernanceSourceProvenance::ReplicaQuery {
                    endpoint: cache.source_endpoint.clone(),
                    fetched_by: cache.fetched_by.clone(),
                };
                if state.is_complete()
                    && state.network() == cache.network
                    && state.governance_canister_id() == cache.metadata.governance_canister_id
                    && state.source() == Some(&source)
                    && state.updated_at() == cache.fetched_at
                    && state.pages_fetched() == cache.completeness.page_count
                    && state.page_size() == cache.completeness.page_size
                    && state.$rows() == cache.completeness.row_count as u64
                {
                    Ok(())
                } else {
                    Err("collection_state disagrees with snapshot identity, provenance, or completeness".to_string())
                }
            })
    }};
}

pub(in crate::nns) use validate_governance_snapshot;

pub(in crate::nns) fn validate_governance_cache_completeness(
    completeness: &CacheCollectionCompleteness,
    row_count: usize,
    point_in_time_error: &str,
) -> Result<(), String> {
    validate_cache_collection_completeness(completeness, row_count)?;
    if completeness.point_in_time_guaranteed {
        return Err(point_in_time_error.to_string());
    }
    Ok(())
}
