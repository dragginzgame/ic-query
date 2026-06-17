//! Module: sns::report::model::reports::proposals::refresh
//!
//! Responsibility: define SNS proposal snapshot refresh report DTOs.
//! Does not own: live proposal paging, cache replacement, or rendering.
//! Boundary: carries refresh result metadata for text and JSON output.

use serde::Serialize;

///
/// SnsProposalsRefreshReport
///
/// Serializable report returned after a complete SNS proposal snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsRefreshReport {
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
    pub proposal_count: usize,
    pub complete: bool,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
}
