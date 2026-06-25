//! Module: sns::report::neurons_cache::model
//!
//! Responsibility: define SNS neuron cache snapshot and projection DTOs.
//! Does not own: cache storage, refresh collection, report rendering, or CLI parsing.
//! Boundary: keeps persisted cache models separate from public report models.

use crate::{
    snapshot_cache::{SnapshotEnvelope, SnapshotHeader},
    sns::report::{SnsNeuronRow, SnsNeuronsSort},
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::PathBuf;

pub(super) type SnsNeuronsCache = SnapshotEnvelope<SnsNeuronsCacheMetadata, SnsNeuronsCacheRows>;

///
/// SnsNeuronsCacheMetadata
///
/// Snapshot metadata identifying the SNS covered by a complete neuron cache.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsCacheMetadata {
    pub(super) sns_wasm_canister_id: String,
    pub(super) id: usize,
    pub(super) name: String,
    pub(super) root_canister_id: String,
    pub(super) governance_canister_id: String,
}

///
/// SnsNeuronsCacheRows
///
/// Snapshot payload containing complete SNS neuron rows.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsCacheRows {
    pub(super) neurons: Vec<SnsNeuronRow>,
}

pub(super) type SnsNeuronsCacheHeader = SnapshotHeader<SnsNeuronsCacheHeaderMetadata>;

///
/// SnsNeuronsCacheHeaderMetadata
///
/// Minimal metadata loaded while scanning neuron cache headers.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
pub(super) struct SnsNeuronsCacheHeaderMetadata {
    pub(super) id: usize,
}

///
/// CompleteSnsNeurons
///
/// Complete in-memory neuron collection produced by refresh paging.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CompleteSnsNeurons {
    pub(super) neurons: Vec<SnsNeuronRow>,
    pub(super) page_count: u32,
    pub(super) last_cursor: Option<String>,
}

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
