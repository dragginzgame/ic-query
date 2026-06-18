//! Module: sns::report::model::reports::neurons::cache
//!
//! Responsibility: local SNS neuron cache inspection report DTOs.
//! Does not own: cache discovery, refresh-attempt reads, or rendering.
//! Boundary: preserves cache summary fields for cache list/status output.

use super::SnsNeuronsRefreshAttemptStatus;
use crate::sns::report::SnsCacheSummarySortKey;
use serde::Serialize;

///
/// SnsNeuronsCacheListReport
///
/// Serializable report listing complete local SNS neuron caches.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub cache_count: usize,
    pub caches: Vec<SnsNeuronsCacheSummary>,
}

///
/// SnsNeuronsCacheStatusReport
///
/// Serializable report describing one expected or discovered SNS neuron cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub input: String,
    pub found: bool,
    pub cache: Option<SnsNeuronsCacheSummary>,
    pub expected_cache_path: Option<String>,
    pub refresh_attempt_path: Option<String>,
    pub latest_attempt: Option<SnsNeuronsRefreshAttemptStatus>,
}

///
/// SnsNeuronsCacheSummary
///
/// Serializable summary of one complete SNS neuron snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsCacheSummary {
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub complete: bool,
    pub row_count: usize,
    pub page_count: u32,
    pub page_size: u32,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub cache_path: String,
    pub refresh_attempt_path: String,
    pub latest_attempt: Option<SnsNeuronsRefreshAttemptStatus>,
}

impl SnsCacheSummarySortKey for SnsNeuronsCacheSummary {
    fn id(&self) -> usize {
        self.id
    }

    fn root_canister_id(&self) -> &str {
        &self.root_canister_id
    }
}
