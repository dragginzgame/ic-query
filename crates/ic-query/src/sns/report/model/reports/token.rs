//! Module: sns::report::model::reports::token
//!
//! Responsibility: SNS token metadata report DTOs.
//! Does not own: ledger calls, token amount formatting, or rendering.
//! Boundary: preserves raw token metadata fields for text and JSON output.

use crate::icrc::{IcrcIndexData, IcrcTokenData};
use serde::Serialize;

///
/// SnsTokenReport
///
/// Serializable report for SNS ledger token metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTokenReport {
    pub schema_version: u32,
    /// Shared record, flattened in JSON.
    #[serde(flatten)]
    pub context: crate::sns::SnsInstanceContext,
    pub ledger_canister_id: String,
    pub sns_index_canister_id: String,
    /// Raw ledger metadata shared with ICRC reports and sources.
    #[serde(flatten)]
    pub token: IcrcTokenData,
    /// Ledger-discovered index principal or its unavailability reason.
    pub ledger_index: IcrcIndexData,
}
