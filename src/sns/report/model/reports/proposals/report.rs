//! Module: sns::report::model::reports::proposals::report
//!
//! Responsibility: define SNS proposal list and detail report DTOs.
//! Does not own: live governance calls, row conversion, cache reads, or rendering.
//! Boundary: preserves report-level fields for text and JSON output.

use super::row::SnsProposalRow;

use serde::Serialize;

///
/// SnsProposalReport
///
/// Serializable report for one SNS governance proposal detail lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalReport {
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
    pub proposal_id: u64,
    pub verbose: bool,
    pub show_ballots: bool,
    pub data_source: String,
    pub cache_path: Option<String>,
    pub cache_complete: Option<bool>,
    pub proposal: SnsProposalRow,
}

///
/// SnsProposalsReport
///
/// Serializable report for a bounded SNS governance proposal listing.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsReport {
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
    pub before_proposal_id: Option<u64>,
    pub status_filter: String,
    pub topic_filter: String,
    pub sort: String,
    pub verbose: bool,
    pub data_source: String,
    pub cache_path: Option<String>,
    pub cache_complete: Option<bool>,
    pub proposal_count: usize,
    pub proposals: Vec<SnsProposalRow>,
}
