//! Module: ic::model::requests::network
//!
//! Responsibility: bounded official Dashboard network-resource request contracts.
//! Does not own: metrics, canister discovery, transport, or reports.
//! Boundary: captures daily-statistics bounds and finite boundary-node resource intent.

use super::IcDashboardRequest;

use serde::Serialize;

///
/// IcDailyStatsQuery
///
/// One explicitly bounded official Dashboard daily-statistics query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcDailyStatsQuery {
    /// Inclusive query start as Unix seconds.
    pub start_unix_secs: u64,
    /// Inclusive query end as Unix seconds.
    pub end_unix_secs: u64,
}

impl IcDailyStatsQuery {
    /// Construct one explicit daily-statistics query window.
    #[must_use]
    pub const fn new(start_unix_secs: u64, end_unix_secs: u64) -> Self {
        Self {
            start_unix_secs,
            end_unix_secs,
        }
    }
}

///
/// IcDailyStatsRequest
///
/// Request accepted by the bounded official Dashboard daily-statistics builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcDailyStatsRequest {
    /// Explicit Dashboard endpoint and caller collection clock.
    pub dashboard: IcDashboardRequest,
    /// Explicitly bounded daily-statistics query.
    pub query: IcDailyStatsQuery,
}

impl IcDailyStatsRequest {
    /// Construct one bounded live Dashboard daily-statistics request.
    #[must_use]
    pub const fn new(dashboard: IcDashboardRequest, query: IcDailyStatsQuery) -> Self {
        Self { dashboard, query }
    }
}
