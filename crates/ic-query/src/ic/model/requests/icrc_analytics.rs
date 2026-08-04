//! Module: ic::model::requests::icrc_analytics
//!
//! Responsibility: bounded official ICRC analytics request contracts.
//! Does not own: native ledger queries, transport, source validation, or reports.
//! Boundary: captures one ledger-scoped total-supply series and explicit time bounds.

use serde::Serialize;

///
/// IcIcrcTotalSupplyQuery
///
/// One explicitly bounded total-supply series query for an ICRC ledger.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcTotalSupplyQuery {
    /// Inclusive query start as Unix seconds.
    pub start_unix_secs: u64,
    /// Inclusive query end as Unix seconds.
    pub end_unix_secs: u64,
    /// Requested observation interval in seconds.
    pub step_secs: u32,
}

impl IcIcrcTotalSupplyQuery {
    /// Construct one explicit total-supply query window.
    #[must_use]
    pub const fn new(start_unix_secs: u64, end_unix_secs: u64, step_secs: u32) -> Self {
        Self {
            start_unix_secs,
            end_unix_secs,
            step_secs,
        }
    }
}

///
/// IcIcrcTotalSupplyRequest
///
/// Request accepted by the bounded official ICRC analytics report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcTotalSupplyRequest {
    /// Official ICRC analytics API base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// ICRC ledger canister principal requested from the analytics service.
    pub ledger_canister_id: String,
    /// Explicitly bounded total-supply query.
    pub query: IcIcrcTotalSupplyQuery,
}

impl IcIcrcTotalSupplyRequest {
    /// Construct one bounded live ICRC total-supply analytics request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        query: IcIcrcTotalSupplyQuery,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
            query,
        }
    }
}
