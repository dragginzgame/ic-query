//! Module: sns::report::model::reports::proposals::cache
//!
//! Responsibility: define SNS proposal cache list and status report DTOs.
//! Does not own: cache discovery, cache file reads, or text rendering.
//! Boundary: preserves cache metadata fields for text and JSON reports.

use super::attempt::SnsProposalsRefreshAttemptStatus;
use crate::sns::report::SnsCacheSummarySortKey;

use serde::Serialize;

///
/// SnsProposalsCacheListReport
///
/// Serializable report listing complete local SNS proposal caches.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub cache_count: usize,
    pub caches: Vec<SnsProposalsCacheSummary>,
}

///
/// SnsProposalsCacheStatusReport
///
/// Serializable report describing one expected or discovered SNS proposal cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub input: String,
    pub found: bool,
    pub cache: Option<SnsProposalsCacheSummary>,
    pub expected_cache_path: Option<String>,
    pub refresh_attempt_path: Option<String>,
    pub latest_attempt: Option<SnsProposalsRefreshAttemptStatus>,
}

///
/// SnsProposalsCacheSummary
///
/// Serializable summary of one complete SNS proposal snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsCacheSummary {
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
    pub latest_attempt: Option<SnsProposalsRefreshAttemptStatus>,
}

impl SnsCacheSummarySortKey for SnsProposalsCacheSummary {
    fn id(&self) -> usize {
        self.id
    }

    fn root_canister_id(&self) -> &str {
        &self.root_canister_id
    }
}
