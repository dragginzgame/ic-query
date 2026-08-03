//! Module: sns::report::model::reports::cache
//!
//! Responsibility: shared SNS complete-snapshot cache inspection DTOs.
//! Does not own: cache discovery, collection-specific storage, or rendering.
//! Boundary: preserves fields common to neuron and proposal cache list/status reports.

use super::SnsRefreshAttemptStatus;
use crate::cache::CacheValidationStatus;
use serde::Serialize;

///
/// SnsCacheListReport
///
/// Serializable report listing one family of complete local SNS caches.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub cache_count: usize,
    pub caches: Vec<SnsCacheSummary>,
}

///
/// SnsCacheStatusReport
///
/// Serializable report describing one expected or discovered SNS cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub input: String,
    pub found: bool,
    pub cache: Option<SnsCacheSummary>,
    pub expected_cache_path: Option<String>,
    pub refresh_attempt_path: Option<String>,
    pub latest_attempt: Option<SnsRefreshAttemptStatus>,
}

///
/// SnsCacheSummary
///
/// Serializable summary of one complete SNS snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCacheSummary {
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub cache_status: CacheValidationStatus,
    pub cache_error: Option<String>,
    pub complete: bool,
    pub row_count: usize,
    pub page_count: u32,
    pub page_size: u32,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub cache_path: String,
    pub refresh_attempt_path: String,
    pub latest_attempt: Option<SnsRefreshAttemptStatus>,
}
