//! Module: nns::proposals::report::cache::model
//!
//! Responsibility: NNS proposal snapshot cache and cache-report DTOs.
//! Does not own: cache file IO, refresh orchestration, or text rendering.
//! Boundary: defines complete proposal snapshot metadata, rows, and reports.

use crate::{
    nns::{
        NnsGovernanceRefreshAttemptStatus, governance::NnsGovernanceCacheMetadata,
        proposals::report::model::NnsProposalRow,
    },
    snapshot_cache::{SnapshotEnvelope, SnapshotRefreshAttempt},
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(super) type NnsProposalCache =
    SnapshotEnvelope<NnsGovernanceCacheMetadata, NnsProposalCacheRows>;

pub(super) const NNS_PROPOSAL_CACHE_FIELDS: &[&str] = &[
    "schema_version",
    "network",
    "source_endpoint",
    "fetched_at",
    "fetched_by",
    "domain",
    "entity",
    "collection",
    "scope",
    "governance_canister_id",
    "completeness",
    "proposals",
];

pub(super) type NnsProposalRefreshAttempt = SnapshotRefreshAttempt<NnsGovernanceCacheMetadata>;

///
/// NnsProposalRefreshReport
///
/// Serializable report for complete NNS proposal snapshot refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub proposal_count: usize,
    pub page_size: u32,
    pub page_count: u32,
    pub complete: bool,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
    pub attempt_finalization_error: Option<String>,
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
pub struct NnsProposalCacheListReport {
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
pub struct NnsProposalCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub found: bool,
    pub cache: Option<NnsProposalCacheSummary>,
    pub expected_cache_path: String,
    pub refresh_attempt_path: String,
    pub latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
}

///
/// NnsProposalCacheSummary
///
/// Serializable summary of one complete NNS proposal snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalCacheSummary {
    pub governance_canister_id: String,
    pub cache_status: String,
    pub cache_error: Option<String>,
    pub complete: bool,
    pub row_count: usize,
    pub page_count: u32,
    pub page_size: u32,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub cache_path: String,
    pub refresh_attempt_path: String,
    pub latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
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
