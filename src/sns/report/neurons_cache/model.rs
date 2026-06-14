use super::super::{SnsNeuronRow, SnsNeuronsSort};
use crate::cache_file::JsonCacheReport;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsCache {
    pub(super) schema_version: u32,
    pub(super) network: String,
    pub(super) sns_wasm_canister_id: String,
    pub(super) fetched_at: String,
    pub(super) source_endpoint: String,
    pub(super) fetched_by: String,
    pub(super) id: usize,
    pub(super) name: String,
    pub(super) root_canister_id: String,
    pub(super) governance_canister_id: String,
    pub(super) completeness: SnsNeuronsCompleteness,
    pub(super) neurons: Vec<SnsNeuronRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsCompleteness {
    pub(super) status: String,
    pub(super) page_size: u32,
    pub(super) page_count: u32,
    pub(super) row_count: usize,
    pub(super) point_in_time_guaranteed: bool,
}

impl JsonCacheReport for SnsNeuronsCache {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
pub(super) struct SnsNeuronsCacheHeader {
    pub(super) schema_version: u32,
    pub(super) network: String,
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
