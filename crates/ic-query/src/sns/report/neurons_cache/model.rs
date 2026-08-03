//! Module: sns::report::neurons_cache::model
//!
//! Responsibility: define SNS neuron cache snapshot and projection DTOs.
//! Does not own: cache storage, refresh collection, report rendering, or CLI parsing.
//! Boundary: keeps persisted cache models separate from public report models.

use crate::{
    snapshot_cache::CompletePagedCollection,
    sns::report::{
        SnsNeuronRow, SnsNeuronsSort, cache_storage::SnsStoredCache,
        neurons_cache::paths::SnsNeuronsCacheCollection,
    },
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::PathBuf;

pub(super) type SnsNeuronsCache = SnsStoredCache<SnsNeuronsCacheCollection>;

pub(super) const SNS_NEURONS_CACHE_FIELDS: &[&str] = &[
    "schema_version",
    "network",
    "source_endpoint",
    "fetched_at",
    "fetched_by",
    "domain",
    "entity",
    "collection",
    "scope",
    "sns_wasm_canister_id",
    "id",
    "name",
    "root_canister_id",
    "governance_canister_id",
    "completeness",
    "neurons",
];

///
/// SnsNeuronsCacheRows
///
/// Snapshot payload containing complete SNS neuron rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsCacheRows {
    pub(super) neurons: Vec<SnsNeuronRow>,
}

///
/// CompleteSnsNeurons
///
/// Complete in-memory neuron collection produced by refresh paging.
///

pub(super) type CompleteSnsNeurons = CompletePagedCollection<SnsNeuronRow>;

///
/// SnsNeuronsCachedReportParts
///
/// Cache-derived inputs needed to assemble an SNS neurons report.
///

pub(super) struct SnsNeuronsCachedReportParts {
    pub(super) requested_limit: u32,
    pub(super) sort: SnsNeuronsSort,
    pub(super) cache: SnsNeuronsCache,
    pub(super) total_neuron_count: usize,
    pub(super) cache_path: PathBuf,
    pub(super) verbose: bool,
}
