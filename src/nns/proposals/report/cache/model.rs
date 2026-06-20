//! Module: nns::proposals::report::cache::model
//!
//! Responsibility: NNS proposal snapshot cache and cache-report DTOs.
//! Does not own: cache file IO, refresh orchestration, or text rendering.
//! Boundary: defines complete proposal snapshot metadata, rows, and reports.

use crate::{
    nns::proposals::report::model::NnsProposalRow,
    snapshot_cache::{SnapshotEnvelope, SnapshotRefreshAttempt},
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::PathBuf;

pub(super) type NnsProposalCache = SnapshotEnvelope<NnsProposalCacheMetadata, NnsProposalCacheRows>;

pub(super) type NnsProposalRefreshAttempt =
    SnapshotRefreshAttempt<NnsProposalRefreshAttemptMetadata>;

///
/// NnsProposalRefreshRequest
///
/// Request accepted by the complete NNS proposal snapshot refresh builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalRefreshRequest {
    pub(in crate::nns::proposals) network: String,
    pub(in crate::nns::proposals) source_endpoint: String,
    pub(in crate::nns::proposals) now_unix_secs: u64,
    pub(in crate::nns::proposals) icp_root: PathBuf,
    pub(in crate::nns::proposals) page_size: u32,
    pub(in crate::nns::proposals) max_pages: Option<u32>,
}

///
/// NnsProposalCacheListRequest
///
/// Request accepted by the local NNS proposal cache list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalCacheListRequest {
    pub(in crate::nns::proposals) network: String,
    pub(in crate::nns::proposals) icp_root: PathBuf,
}

///
/// NnsProposalCacheStatusRequest
///
/// Request accepted by the local NNS proposal cache status report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalCacheStatusRequest {
    pub(in crate::nns::proposals) network: String,
    pub(in crate::nns::proposals) icp_root: PathBuf,
}

///
/// NnsProposalRefreshReport
///
/// Serializable report for complete NNS proposal snapshot refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub proposal_count: usize,
    pub page_size: u32,
    pub page_count: u32,
    pub complete: bool,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub cache_path: String,
    pub refresh_attempt_path: String,
    pub refresh_lock_path: String,
}

///
/// NnsProposalCacheListReport
///
/// Serializable report listing local complete NNS proposal caches.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub cache_count: usize,
    pub caches: Vec<NnsProposalCacheSummary>,
}

///
/// NnsProposalCacheStatusReport
///
/// Serializable report describing the NNS proposal cache and latest attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub found: bool,
    pub cache: Option<NnsProposalCacheSummary>,
    pub expected_cache_path: String,
    pub refresh_attempt_path: String,
    pub latest_attempt: Option<NnsProposalRefreshAttemptStatus>,
}

///
/// NnsProposalCacheSummary
///
/// Serializable summary of one complete NNS proposal snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalCacheSummary {
    pub governance_canister_id: String,
    pub complete: bool,
    pub row_count: usize,
    pub page_count: u32,
    pub page_size: u32,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub cache_path: String,
    pub refresh_attempt_path: String,
    pub latest_attempt: Option<NnsProposalRefreshAttemptStatus>,
}

///
/// NnsProposalRefreshAttemptStatus
///
/// Serializable status for the latest NNS proposal snapshot refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalRefreshAttemptStatus {
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub page_size: u32,
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
    pub last_error: Option<String>,
}

impl From<NnsProposalRefreshAttempt> for NnsProposalRefreshAttemptStatus {
    fn from(attempt: NnsProposalRefreshAttempt) -> Self {
        Self {
            status: attempt.status,
            started_at: attempt.started_at,
            updated_at: attempt.updated_at,
            page_size: attempt.page_size,
            pages_fetched: attempt.pages_fetched,
            rows_fetched: attempt.rows_fetched,
            last_cursor: attempt.last_cursor,
            last_error: attempt.last_error,
        }
    }
}

///
/// NnsProposalCacheMetadata
///
/// Snapshot metadata identifying the NNS governance canister.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct NnsProposalCacheMetadata {
    pub(super) governance_canister_id: String,
}

///
/// NnsProposalCacheRows
///
/// Snapshot payload containing complete NNS proposal rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct NnsProposalCacheRows {
    pub(super) proposals: Vec<NnsProposalRow>,
}

///
/// NnsProposalRefreshAttemptMetadata
///
/// Refresh-attempt metadata identifying the NNS governance canister.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct NnsProposalRefreshAttemptMetadata {
    pub(super) governance_canister_id: String,
}

///
/// CompleteNnsProposalCollection
///
/// Complete in-memory proposal collection produced by refresh paging.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CompleteNnsProposalCollection {
    pub(super) proposals: Vec<NnsProposalRow>,
    pub(super) page_count: u32,
    pub(super) last_cursor: Option<String>,
}
