//! Module: sns::report::model::reports::token
//!
//! Responsibility: SNS token metadata report DTOs.
//! Does not own: ledger calls, token amount formatting, or rendering.
//! Boundary: preserves raw token metadata fields for text and JSON output.

use crate::icrc::IcrcTokenData;
use serde::Serialize;

///
/// SnsTokenReport
///
/// Serializable report for SNS ledger token metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTokenReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub ledger_canister_id: String,
    pub sns_index_canister_id: String,
    /// Raw ledger metadata shared with ICRC reports and sources.
    #[serde(flatten)]
    pub token: IcrcTokenData,
    pub ledger_index_canister_id: Option<String>,
    pub ledger_index_error: Option<String>,
}
