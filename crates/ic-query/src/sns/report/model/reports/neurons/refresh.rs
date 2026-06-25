//! Module: sns::report::model::reports::neurons::refresh
//!
//! Responsibility: complete SNS neuron refresh report DTO.
//! Does not own: refresh orchestration, cache writes, or rendering.
//! Boundary: records the published snapshot and refresh paths for output.

use serde::Serialize;

///
/// SnsNeuronsRefreshReport
///
/// Serializable report returned after a complete SNS neuron snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsRefreshReport {
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
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub refresh_attempt_path: String,
    pub page_size: u32,
    pub page_count: u32,
    pub neuron_count: usize,
    pub complete: bool,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
}
