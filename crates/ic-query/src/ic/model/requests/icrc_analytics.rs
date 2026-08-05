//! Module: ic::model::requests::icrc_analytics
//!
//! Responsibility: bounded official ICRC analytics request contracts.
//! Does not own: native ledger queries, transport, source validation, or reports.
//! Boundary: shares one ledger target while keeping series bounds operation-specific.

use serde::Serialize;
use std::fmt;

///
/// IcIcrcAnalyticsRequest
///
/// Shared endpoint, collection time, and ledger target for official ICRC analytics.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcAnalyticsRequest {
    /// Official ICRC analytics API base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// ICRC ledger canister principal requested from the analytics service.
    pub ledger_canister_id: String,
}

impl IcIcrcAnalyticsRequest {
    /// Construct one official ICRC analytics target.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            ledger_canister_id: ledger_canister_id.into(),
        }
    }
}

///
/// IcIcrcIndexedCountKind
///
/// Scalar ledger resource counted by the official ICRC analytics index.
///

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IcIcrcIndexedCountKind {
    /// Accounts represented by the index.
    Account,
    /// Holders represented by the index.
    Holder,
    /// Transactions represented by the index.
    Transaction,
}

impl IcIcrcIndexedCountKind {
    /// Return the stable singular resource label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Holder => "holder",
            Self::Transaction => "transaction",
        }
    }

    #[cfg(feature = "dashboard-host")]
    pub(crate) const fn resource_path_segment(self) -> &'static str {
        match self {
            Self::Account => "accounts",
            Self::Holder => "holders",
            Self::Transaction => "transactions",
        }
    }
}

impl fmt::Display for IcIcrcIndexedCountKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// IcIcrcIndexedCountRequest
///
/// Request for one scalar count from the official ICRC analytics index.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcIndexedCountRequest {
    /// Shared analytics endpoint, collection time, and ledger identity.
    pub analytics: IcIcrcAnalyticsRequest,
    /// Indexed resource to count.
    pub kind: IcIcrcIndexedCountKind,
}

impl IcIcrcIndexedCountRequest {
    /// Construct one live indexed-count request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        kind: IcIcrcIndexedCountKind,
    ) -> Self {
        Self {
            analytics: IcIcrcAnalyticsRequest::new(
                source_endpoint,
                now_unix_secs,
                ledger_canister_id,
            ),
            kind,
        }
    }
}

///
/// IcIcrcTokenValueQuery
///
/// One explicitly bounded token-value series query for an ICRC ledger.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcTokenValueQuery {
    /// Query start as Unix seconds.
    pub start_unix_secs: u64,
    /// Query end as Unix seconds.
    pub end_unix_secs: u64,
    /// Maximum rows requested from the official API.
    pub limit: u16,
}

impl IcIcrcTokenValueQuery {
    /// Construct one explicit token-value query window.
    #[must_use]
    pub const fn new(start_unix_secs: u64, end_unix_secs: u64, limit: u16) -> Self {
        Self {
            start_unix_secs,
            end_unix_secs,
            limit,
        }
    }
}

///
/// IcIcrcTokenValueRequest
///
/// Request accepted by the bounded official ICRC token-value report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcTokenValueRequest {
    /// Shared analytics endpoint, collection time, and ledger identity.
    pub analytics: IcIcrcAnalyticsRequest,
    /// Explicitly bounded token-value query.
    pub query: IcIcrcTokenValueQuery,
}

impl IcIcrcTokenValueRequest {
    /// Construct one bounded live ICRC token-value request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        query: IcIcrcTokenValueQuery,
    ) -> Self {
        Self {
            analytics: IcIcrcAnalyticsRequest::new(
                source_endpoint,
                now_unix_secs,
                ledger_canister_id,
            ),
            query,
        }
    }
}

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
    /// Shared analytics endpoint, collection time, and ledger identity.
    pub analytics: IcIcrcAnalyticsRequest,
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
            analytics: IcIcrcAnalyticsRequest::new(
                source_endpoint,
                now_unix_secs,
                ledger_canister_id,
            ),
            query,
        }
    }
}
