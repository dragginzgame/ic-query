//! Module: sns::report::model::reports::neurons::report
//!
//! Responsibility: SNS neuron listing report DTO.
//! Does not own: live fetching, cache-backed sorting, or rendering.
//! Boundary: preserves neuron listing metadata and rows for text and JSON.

use super::row::SnsNeuronRow;
use serde::Serialize;

///
/// SnsNeuronsReport
///
/// Serializable report for bounded or cache-backed SNS neuron listings.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub requested_limit: u32,
    pub owner_principal_id: Option<String>,
    pub verbose: bool,
    pub data_source: String,
    pub sort: String,
    pub cache_path: Option<String>,
    pub cache_complete: Option<bool>,
    pub total_neuron_count: usize,
    pub neuron_count: usize,
    pub neurons: Vec<SnsNeuronRow>,
}
