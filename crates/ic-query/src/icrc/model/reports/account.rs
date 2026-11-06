//! Module: icrc::model::reports::account
//!
//! Responsibility: serialized ICRC balance and allowance report contracts.
//! Does not own: account history, ledger-wide reports, live transport, or rendering.
//! Boundary: preserves raw account and allowance result fields.

use serde::Serialize;

///
/// IcrcBalanceData
///
/// Raw balance result shared by sources and reports plus enough token metadata for display.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcBalanceData {
    /// Ledger token symbol.
    pub token_symbol: String,
    /// Ledger token decimal places.
    pub decimals: u8,
    /// Balance in raw decimal base units.
    pub balance: String,
}

///
/// IcrcAllowanceData
///
/// Raw allowance result shared by sources and reports plus enough token metadata for display.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAllowanceData {
    /// Ledger token symbol.
    pub token_symbol: String,
    /// Ledger token decimal places.
    pub decimals: u8,
    /// Allowance in raw decimal base units.
    pub allowance: String,
    /// Optional expiry in raw decimal Unix nanoseconds.
    pub expires_at_unix_nanos: Option<String>,
}

///
/// IcrcBalanceReport
///
/// Serializable report for one generic ICRC account balance lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcBalanceReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    pub account_owner: String,
    pub subaccount_hex: Option<String>,
    /// Raw balance and token display metadata.
    #[serde(flatten)]
    pub value: IcrcBalanceData,
}

///
/// IcrcAllowanceReport
///
/// Serializable report for one generic ICRC allowance lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcrcAllowanceReport {
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: super::IcrcReportProvenance,
    pub account_owner: String,
    pub account_subaccount_hex: Option<String>,
    pub spender_owner: String,
    pub spender_subaccount_hex: Option<String>,
    /// Raw allowance, expiry and token display metadata.
    #[serde(flatten)]
    pub value: IcrcAllowanceData,
}
