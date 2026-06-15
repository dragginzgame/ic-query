use super::attempt::SnsNeuronsRefreshAttemptStatus;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub cache_count: usize,
    pub caches: Vec<SnsNeuronsCacheSummary>,
}

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
