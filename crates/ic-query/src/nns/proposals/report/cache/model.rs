//! Module: nns::proposals::report::cache::model
//!
//! Responsibility: NNS proposal snapshot cache and cache-report DTOs.
//! Does not own: cache file IO, refresh orchestration, or text rendering.
//! Boundary: defines complete proposal snapshot metadata, rows, and reports.

use crate::{
    nns::{
        NnsGovernanceRefreshAttemptStatus,
        governance::{NnsGovernanceCacheMetadata, NnsGovernanceCacheSummary},
        proposals::report::{collection::NnsProposalCollectionState, model::NnsProposalRow},
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
    "collection_state",
    "proposals",
];

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
    pub caches: Vec<NnsGovernanceCacheSummary>,
    /// Collection inspected by this proposal-only list operation.
    pub collection: String,
    /// Expected refresh-attempt sidecar path.
    pub refresh_attempt_path: String,
    /// Latest readable attempt for this single expected snapshot.
    pub latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
}

///
/// NnsProposalCacheRows
///
/// Snapshot payload containing complete NNS proposal rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct NnsProposalCacheRows {
    pub(super) collection_state: NnsProposalCollectionState,
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
    pub(super) state: NnsProposalCollectionState,
}
