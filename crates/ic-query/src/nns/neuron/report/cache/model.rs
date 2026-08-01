//! Module: nns::neuron::report::cache::model
//!
//! Responsibility: NNS neuron snapshot cache and cache-report DTOs.
//! Does not own: cache file IO, refresh orchestration, or text rendering.
//! Boundary: defines complete public neuron snapshot rows and report shapes.

use crate::{
    nns::{
        NnsGovernanceRefreshAttemptStatus, governance::NnsGovernanceCacheMetadata,
        neuron::report::model::NnsNeuronRow,
    },
    snapshot_cache::SnapshotEnvelope,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};

pub(super) type NnsNeuronCache = SnapshotEnvelope<NnsGovernanceCacheMetadata, NnsNeuronCacheRows>;

pub(super) const NNS_NEURON_CACHE_FIELDS: &[&str] = &[
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
    "neurons",
];

///
/// NnsNeuronRefreshReport
///
/// Serializable outcome of a complete public neuron-index refresh.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronRefreshReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Refreshed network identity.
    pub network: String,
    /// NNS Governance canister principal.
    pub governance_canister_id: String,
    /// Number of public neuron rows published.
    pub neuron_count: usize,
    /// Page size used for the walk.
    pub page_size: u32,
    /// Pages fetched through API exhaustion.
    pub page_count: u32,
    /// Whether the published collection is complete.
    pub complete: bool,
    /// Whether every row is guaranteed to describe one Governance instant.
    pub point_in_time_guaranteed: bool,
    /// Whether a previous complete cache was replaced.
    pub replaced_existing_cache: bool,
    /// Failure to finalize attempt metadata after successful publication.
    pub attempt_finalization_error: Option<String>,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Replica endpoint used for every page.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Published snapshot path.
    pub cache_path: String,
    /// Refresh-attempt sidecar path.
    pub refresh_attempt_path: String,
    /// Refresh-lock path.
    pub refresh_lock_path: String,
}

///
/// NnsNeuronCacheStatusReport
///
/// Serializable local status of the complete NNS neuron snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronCacheStatusReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Cache network namespace.
    pub network: String,
    /// Directory containing the NNS neuron collection.
    pub cache_root: String,
    /// Whether the expected snapshot path exists.
    pub found: bool,
    /// Valid or invalid snapshot summary when the path exists.
    pub cache: Option<NnsNeuronCacheSummary>,
    /// Expected complete snapshot path.
    pub expected_cache_path: String,
    /// Expected refresh-attempt path.
    pub refresh_attempt_path: String,
    /// Latest valid refresh-attempt evidence.
    pub latest_attempt: Option<NnsGovernanceRefreshAttemptStatus>,
}

///
/// NnsNeuronCacheSummary
///
/// Serializable summary of one complete or invalid NNS neuron snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsNeuronCacheSummary {
    /// Cache validation status.
    pub cache_status: String,
    /// Validation error for an invalid cache.
    pub cache_error: Option<String>,
    /// Whether API exhaustion was proven.
    pub complete: bool,
    /// Whether every row is guaranteed to describe one Governance instant.
    pub point_in_time_guaranteed: bool,
    /// Stored public neuron row count.
    pub row_count: usize,
    /// Stored page count.
    pub page_count: u32,
    /// Stored page size.
    pub page_size: u32,
    /// Snapshot collection timestamp.
    pub fetched_at: String,
    /// Snapshot source endpoint.
    pub source_endpoint: String,
    /// Complete snapshot path.
    pub cache_path: String,
}

///
/// NnsNeuronCacheRows
///
/// Complete public neuron rows stored in one snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct NnsNeuronCacheRows {
    pub(super) neurons: Vec<NnsNeuronRow>,
}

///
/// CompleteNeuronCollection
///
/// Complete in-memory neuron collection produced by refresh paging.
///

pub(super) struct CompleteNeuronCollection {
    pub(super) neurons: Vec<NnsNeuronRow>,
    pub(super) page_count: u32,
    pub(super) last_cursor: Option<String>,
}
