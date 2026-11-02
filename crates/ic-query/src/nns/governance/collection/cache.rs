//! Shared status and summary projection for native Governance collection caches.

use super::{
    NnsGovernanceAttemptReadError, NnsGovernanceCacheMetadata, NnsGovernanceCacheRequest,
    read_governance_refresh_attempt_status,
};
use crate::{
    HostCacheError,
    cache::{
        CacheCollectionCompleteness, CacheValidationStatus, validate_cache_collection_completeness,
    },
    cache_file::open_managed_file,
    snapshot_cache::{SnapshotEnvelope, SnapshotJsonPaths},
};
use serde::Serialize;
use std::{fs::File, path::Path};

pub(in crate::nns) fn build_governance_cache_status_report<Error>(
    request: &NnsGovernanceCacheRequest,
    collection: crate::nns::governance::NnsGovernanceCollection,
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
    /// Path of the inspected snapshot, including invalid content.
    pub cache_path: String,
    /// Admitted facts or original refusal evidence.
    #[serde(flatten)]
    pub admission: NnsGovernanceCacheAdmission,
}

///
/// NnsGovernanceCacheAdmission
///
/// Complete Governance collection facts or its admission failure.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "cache_status", rename_all = "snake_case")]
pub enum NnsGovernanceCacheAdmission {
    /// An admitted complete collection.
    Valid {
        /// Admitted complete collection facts.
        #[serde(flatten)]
        data: NnsGovernanceValidCacheSummary,
    },
    /// Existing content that failed admission.
    Invalid {
        /// Original admission failure.
        cache_error: String,
    },
}

///
/// NnsGovernanceValidCacheSummary
///
/// Collection facts read only from an admitted complete Governance snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsGovernanceValidCacheSummary {
    /// Admitted Governance principal.
    pub governance_canister_id: String,
    /// Source guarantee retained from the admitted snapshot.
    pub point_in_time_guaranteed: bool,
    /// Rows in the admitted complete collection.
    pub row_count: usize,
    /// Pages collected through API exhaustion.
    pub page_count: u32,
    /// Requested rows per source page.
    pub page_size: u32,
    /// Admitted collection timestamp.
    pub fetched_at: String,
    /// Explicit endpoint that supplied the admitted snapshot.
    pub source_endpoint: String,
}

impl NnsGovernanceCacheSummary {
    /// Return verified facts only after complete-snapshot admission.
    #[must_use]
    pub const fn valid(&self) -> Option<&NnsGovernanceValidCacheSummary> {
        match &self.admission {
            NnsGovernanceCacheAdmission::Valid { data } => Some(data),
            NnsGovernanceCacheAdmission::Invalid { .. } => None,
        }
    }

    /// Return the cache admission outcome.
    #[must_use]
    pub const fn cache_status(&self) -> CacheValidationStatus {
        match self.admission {
            NnsGovernanceCacheAdmission::Valid { .. } => CacheValidationStatus::Valid,
            NnsGovernanceCacheAdmission::Invalid { .. } => CacheValidationStatus::Invalid,
        }
    }

    /// Retain the original content refusal.
    #[must_use]
    pub fn cache_error(&self) -> Option<&str> {
        match &self.admission {
            NnsGovernanceCacheAdmission::Invalid { cache_error } => Some(cache_error),
            NnsGovernanceCacheAdmission::Valid { .. } => None,
        }
    }

    /// Project a snapshot after family identity, row and completeness admission.
    pub(in crate::nns) fn from_snapshot<Rows>(
        path: &Path,
        cache: &SnapshotEnvelope<NnsGovernanceCacheMetadata, Rows>,
    ) -> Self {
        Self {
            cache_path: path.display().to_string(),
            admission: NnsGovernanceCacheAdmission::Valid {
                data: NnsGovernanceValidCacheSummary {
                    governance_canister_id: cache.metadata.governance_canister_id.clone(),
                    point_in_time_guaranteed: cache.completeness.point_in_time_guaranteed,
                    row_count: cache.completeness.row_count,
                    page_count: cache.completeness.page_count,
                    page_size: cache.completeness.page_size,
                    fetched_at: cache.fetched_at.clone(),
                    source_endpoint: cache.source_endpoint.clone(),
                },
            },
        }
    }

    /// Retain refusal evidence without projecting unvalidated snapshot values.
    pub(in crate::nns) fn invalid(path: &Path, reason: String) -> Self {
        Self {
            cache_path: path.display().to_string(),
            admission: NnsGovernanceCacheAdmission::Invalid {
                cache_error: reason,
            },
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
    pub collection: crate::nns::governance::NnsGovernanceCollection,
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
    pub latest_attempt: Option<crate::cache::SnapshotRefreshAttemptStatus>,
}

impl NnsGovernanceCacheStatusReport {
    /// Assemble status from family-owned loading and attempt admission.
    pub(in crate::nns) fn from_cache(
        request: &NnsGovernanceCacheRequest,
        collection: crate::nns::governance::NnsGovernanceCollection,
        paths: &SnapshotJsonPaths,
        cache: Option<NnsGovernanceCacheSummary>,
        latest_attempt: Option<crate::cache::SnapshotRefreshAttemptStatus>,
    ) -> Self {
        Self {
            schema_version: 1,
            network: request.network.clone(),
            collection,
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
