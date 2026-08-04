//! Module: sns::report::model::reports::list
//!
//! Responsibility: deployed SNS list and info report DTOs.
//! Does not own: SNS-W fetching, metadata lookup, sorting, or rendering.
//! Boundary: preserves raw report fields for text and JSON output writers.

use crate::report::ReportDataSource;
use serde::Serialize;

///
/// SnsListReport
///
/// Serializable report for deployed SNS listings.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    /// Whether the rows came from a live collection or the joined catalog cache.
    pub data_source: ReportDataSource,
    /// Complete snapshot path when `data_source` is `cache`.
    pub cache_path: Option<String>,
    /// Whether the cache provenance represents an API-exhausted complete snapshot.
    pub cache_complete: Option<bool>,
    /// Whether the view includes SNS instances outside the normal visible lifecycle set.
    pub all_lifecycles: bool,
    pub verbose: bool,
    pub sort: String,
    /// Number of SNS instances in the complete joined catalog before view filtering.
    pub catalog_sns_count: usize,
    /// Number of catalog rows excluded by lifecycle filtering.
    pub excluded_sns_count: usize,
    pub sns_count: usize,
    pub metadata_error_count: usize,
    /// Number of returned rows carrying a bounded lifecycle query error.
    pub lifecycle_error_count: usize,
    pub sns_instances: Vec<SnsListRow>,
}

///
/// SnsListRow
///
/// Serializable row for one deployed SNS in a list report.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListRow {
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
    /// Native Swap lifecycle discriminant, when the lifecycle query succeeded.
    pub lifecycle: Option<i32>,
    /// Stable native lifecycle label derived from `lifecycle`.
    pub lifecycle_name: Option<String>,
    /// Bounded lifecycle query failure retained instead of dropping the SNS row.
    pub lifecycle_error: Option<String>,
}

///
/// SnsInfoReport
///
/// Serializable report for one deployed SNS resolved by id or root principal.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsInfoReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}
