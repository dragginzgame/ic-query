//! Module: sns::report::model::reports::token
//!
//! Responsibility: SNS token metadata report DTOs.
//! Does not own: ledger calls, token amount formatting, or rendering.
//! Boundary: preserves raw token metadata fields for text and JSON output.

use serde::Serialize;
use serde_json::Value as JsonValue;

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
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub transfer_fee: String,
    pub total_supply: String,
    pub minting_account_owner: Option<String>,
    pub minting_account_subaccount_hex: Option<String>,
    pub ledger_index_canister_id: Option<String>,
    pub ledger_index_error: Option<String>,
    pub supported_standards: Vec<SnsTokenStandardRow>,
    pub metadata: Vec<SnsTokenMetadataRow>,
}

///
/// SnsTokenStandardRow
///
/// Serializable row for one token standard supported by an SNS ledger.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTokenStandardRow {
    pub name: String,
    pub url: String,
}

///
/// SnsTokenMetadataRow
///
/// Serializable row for one raw SNS ledger metadata entry.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTokenMetadataRow {
    pub key: String,
    pub value_type: String,
    pub value: JsonValue,
}
