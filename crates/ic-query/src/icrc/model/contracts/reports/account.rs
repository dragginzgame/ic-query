//! Module: icrc::model::contracts::reports::account
//!
//! Responsibility: serialized ICRC balance and allowance report contracts.
//! Does not own: account history, ledger-wide reports, live transport, or rendering.
//! Boundary: preserves raw account and allowance result fields.

use serde::Serialize;

///
/// IcrcBalanceReport
///
/// Serializable report for one generic ICRC account balance lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcBalanceReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub account_owner: String,
    pub subaccount_hex: Option<String>,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub balance: String,
}

///
/// IcrcAllowanceReport
///
/// Serializable report for one generic ICRC allowance lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAllowanceReport {
    pub schema_version: u32,
    pub ledger_canister_id: String,
    pub account_owner: String,
    pub account_subaccount_hex: Option<String>,
    pub spender_owner: String,
    pub spender_subaccount_hex: Option<String>,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub allowance: String,
    pub expires_at_unix_nanos: Option<String>,
}
