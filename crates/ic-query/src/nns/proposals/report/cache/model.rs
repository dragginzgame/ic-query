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
#[serde(deny_unknown_fields)]
pub(super) struct NnsProposalCacheRows {
    pub(super) collection_state: NnsProposalCollectionState,
    pub(super) proposals: Vec<NnsProposalRow>,
}
