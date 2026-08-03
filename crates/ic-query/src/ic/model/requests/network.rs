//! Module: ic::model::requests::network
//!
//! Responsibility: bounded official Dashboard network-resource request contracts.
//! Does not own: metrics, canister discovery, transport, or reports.
//! Boundary: captures daily-statistics bounds and finite boundary-node resource intent.

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
    /// Dashboard API v3 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Explicitly bounded daily-statistics query.
    pub query: IcDailyStatsQuery,
}

impl IcDailyStatsRequest {
    /// Construct one bounded live Dashboard daily-statistics request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        query: IcDailyStatsQuery,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            query,
        }
    }
}

///
/// IcBoundaryNodeDataCentersRequest
///
/// Request accepted by the official Dashboard boundary-node data-center builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcBoundaryNodeDataCentersRequest {
    /// Dashboard API v4 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
}

impl IcBoundaryNodeDataCentersRequest {
    /// Construct one live Dashboard boundary-node data-center request.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
        }
    }
}
