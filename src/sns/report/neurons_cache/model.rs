use super::super::{SnsNeuronRow, SnsNeuronsSort};
use crate::snapshot_cache::{SnapshotEnvelope, SnapshotHeader};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::PathBuf;

pub(super) type SnsNeuronsCache = SnapshotEnvelope<SnsNeuronsCacheMetadata, SnsNeuronsCacheRows>;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsCacheMetadata {
    pub(super) sns_wasm_canister_id: String,
    pub(super) id: usize,
    pub(super) name: String,
    pub(super) root_canister_id: String,
    pub(super) governance_canister_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsCacheRows {
    pub(super) neurons: Vec<SnsNeuronRow>,
}

pub(super) type SnsNeuronsCacheHeader = SnapshotHeader<SnsNeuronsCacheHeaderMetadata>;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
pub(super) struct SnsNeuronsCacheHeaderMetadata {
    pub(super) id: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CompleteSnsNeurons {
    pub(super) neurons: Vec<SnsNeuronRow>,
    pub(super) page_count: u32,
    pub(super) last_cursor: Option<String>,
}

pub(super) struct SnsNeuronsCachedReportParts {
    pub(super) requested_limit: u32,
    pub(super) sort: SnsNeuronsSort,
    pub(super) cache: SnsNeuronsCache,
    pub(super) total_neuron_count: usize,
    pub(super) cache_path: PathBuf,
    pub(super) verbose: bool,
}
