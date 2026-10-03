//! Module: nns::proposals::report::cache::model
//!
//! Responsibility: NNS proposal snapshot cache and cache-report DTOs.
//! Does not own: cache file IO, refresh orchestration, or text rendering.
//! Boundary: defines complete proposal snapshot metadata, rows, and reports.

use crate::{
    cache::CacheValidationStatus,
    nns::{
        NnsGovernanceRefreshAttemptStatus, governance::NnsGovernanceCacheMetadata,
        proposals::report::model::NnsProposalRow,
    },
    snapshot_cache::SnapshotEnvelope,
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

///
/// NnsProposalRefreshReport
///
/// Serializable report for complete NNS proposal snapshot refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalRefreshReport {
    /// Version of this report contract; currently `1`.
    pub schema_version: u32,
    /// Network identity; supported NNS reports use `ic`.
    pub network: String,
    /// Canonical mainnet Governance canister principal.
    pub governance_canister_id: String,
    /// Number of proposal rows returned in this report.
    pub proposal_count: usize,
    /// Maximum proposal rows requested in each collection page.
    pub page_size: u32,
    /// Number of source pages admitted by the collection.
    pub page_count: u32,
    /// Whether API exhaustion established a complete sequential collection.
    pub complete: bool,
    /// Whether publication replaced a previously existing snapshot file.
    pub replaced_existing_cache: bool,
    /// Whether the complete snapshot was successfully published.
    pub wrote_cache: bool,
    /// Failure to finalize the attempt sidecar after snapshot publication, when present.
    pub attempt_finalization_error: Option<String>,
    /// UTC collection timestamp retained with the snapshot.
    pub fetched_at: String,
    /// Exact replica endpoint used to collect the snapshot.
    pub source_endpoint: String,
    /// Collector identity recorded in snapshot provenance.
    pub fetched_by: String,
    /// Path of the complete proposal snapshot inspected or published.
    pub cache_path: String,
    /// Path of the latest refresh-attempt sidecar.
    pub refresh_attempt_path: String,
    /// Path of the lock guarding complete snapshot publication.
    pub refresh_lock_path: String,
}

///
/// NnsProposalCacheListReport
///
/// Serializable report listing local complete NNS proposal caches.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalCacheListReport {
    /// Version of this report contract; currently `1`.
    pub schema_version: u32,
    /// Network identity; supported NNS reports use `ic`.
    pub network: String,
    /// Root directory selected for local snapshot inspection.
    pub cache_root: String,
    /// Number of snapshot summaries returned by this inspection.
    pub cache_count: usize,
    /// Local snapshot summaries discovered beneath the selected root.
    pub caches: Vec<NnsProposalCacheSummary>,
}

///
/// NnsProposalCacheStatusReport
///
/// Serializable report describing the NNS proposal cache and latest attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalCacheStatusReport {
    /// Version of this report contract; currently `1`.
    pub schema_version: u32,
    /// Network identity; supported NNS reports use `ic`.
    pub network: String,
    /// Root directory selected for local snapshot inspection.
    pub cache_root: String,
    /// Whether the expected snapshot file exists, including an invalid file.
    pub found: bool,
    /// Inspection summary when the expected snapshot exists.
    pub cache: Option<NnsProposalCacheSummary>,
    /// Expected full-collection snapshot path for this network.
    pub expected_cache_path: String,
    /// Path of the latest refresh-attempt sidecar.
    pub refresh_attempt_path: String,
    /// Latest retained refresh attempt, when a readable sidecar exists.
    pub latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
}

///
/// NnsProposalCacheSummary
///
/// Serializable summary of one complete NNS proposal snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsProposalCacheSummary {
    /// Canonical mainnet Governance canister principal.
    pub governance_canister_id: String,
    /// Result of validating the snapshot against the current contract.
    pub cache_status: CacheValidationStatus,
    /// Validation diagnostic when the snapshot is invalid.
    pub cache_error: Option<String>,
    /// Whether API exhaustion established a complete sequential collection.
    pub complete: bool,
    /// Number of proposal rows retained in the inspected snapshot.
    pub row_count: usize,
    /// Number of source pages admitted by the collection.
    pub page_count: u32,
    /// Maximum proposal rows requested in each collection page.
    pub page_size: u32,
    /// UTC collection timestamp retained with the snapshot.
    pub fetched_at: String,
    /// Exact replica endpoint used to collect the snapshot.
    pub source_endpoint: String,
    /// Path of the complete proposal snapshot inspected or published.
    pub cache_path: String,
    /// Path of the latest refresh-attempt sidecar.
    pub refresh_attempt_path: String,
    /// Latest retained refresh attempt, when a readable sidecar exists.
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
