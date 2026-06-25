//! Module: sns::report::model::reports::list
//!
//! Responsibility: deployed SNS list and info report DTOs.
//! Does not own: SNS-W fetching, metadata lookup, sorting, or rendering.
//! Boundary: preserves raw report fields for text and JSON output writers.

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
    pub verbose: bool,
    pub sort: String,
    pub sns_count: usize,
    pub metadata_error_count: usize,
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
