//! Module: sns::report::model::reports::list
//!
//! Responsibility: deployed SNS list and info report DTOs.
//! Does not own: SNS-W fetching, metadata lookup, sorting, or rendering.
//! Boundary: preserves raw report fields for text and JSON output writers.

use crate::report::ReportProvenance;
use crate::sns::report::model::MainnetSnsCanisters;
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
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: ReportProvenance,
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
    /// Complete deployed-SNS canister identity, serialized as the existing flat fields.
    #[serde(flatten)]
    pub canisters: MainnetSnsCanisters,
    pub metadata_error: Option<String>,
    /// Native Swap lifecycle discriminant, when the lifecycle query succeeded.
    pub lifecycle: Option<i32>,
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
    /// Complete deployed-SNS canister identity, serialized as the existing flat fields.
    #[serde(flatten)]
    pub canisters: MainnetSnsCanisters,
    pub metadata_error: Option<String>,
}
