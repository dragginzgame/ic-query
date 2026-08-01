//! Module: ic::model::requests
//!
//! Responsibility: public IC Dashboard request, query, filter, and selector contracts.
//! Does not own: reports, host source data, errors, transport, or projection.
//! Boundary: defines bounded caller intent without performing live work.

use serde::Serialize;
use std::{fmt, str::FromStr};

///
/// IcMetricKind
///
/// One bounded network metric exposed by the official Dashboard Metrics API.
///

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IcMetricKind {
    /// Network instruction execution rate.
    InstructionRate,
    /// Network message execution rate.
    MessageExecutionRate,
    /// Network cycle burn rate.
    CycleBurnRate,
    /// Network block ingestion rate.
    BlockRate,
    /// Total and currently up node counts.
    IcNodeCount,
    /// Total Subnet count.
    IcSubnetTotal,
    /// Running and stopped canister counts.
    RegisteredCanistersCount,
    /// Total estimated IC energy-consumption rate in kWh.
    TotalIcEnergyConsumptionRateKwh,
    /// Active boundary-node count.
    BoundaryNodesCount,
}

impl IcMetricKind {
    /// Return every metric supported by the bounded report adapter.
    #[must_use]
    pub const fn all() -> [Self; 9] {
        [
            Self::InstructionRate,
            Self::MessageExecutionRate,
            Self::CycleBurnRate,
            Self::BlockRate,
            Self::IcNodeCount,
            Self::IcSubnetTotal,
            Self::RegisteredCanistersCount,
            Self::TotalIcEnergyConsumptionRateKwh,
            Self::BoundaryNodesCount,
        ]
    }

    /// Return the official Dashboard Metrics API path name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InstructionRate => "instruction-rate",
            Self::MessageExecutionRate => "message-execution-rate",
            Self::CycleBurnRate => "cycle-burn-rate",
            Self::BlockRate => "block-rate",
            Self::IcNodeCount => "ic-node-count",
            Self::IcSubnetTotal => "ic-subnet-total",
            Self::RegisteredCanistersCount => "registered-canisters-count",
            Self::TotalIcEnergyConsumptionRateKwh => "total-ic-energy-consumption-rate-kwh",
            Self::BoundaryNodesCount => "boundary-nodes-count",
        }
    }

    #[cfg(feature = "host")]
    pub(crate) const fn series_names(self) -> &'static [&'static str] {
        match self {
            Self::InstructionRate => &["instruction_rate"],
            Self::MessageExecutionRate => &["message_execution_rate"],
            Self::CycleBurnRate => &["cycle_burn_rate"],
            Self::BlockRate => &["block_rate"],
            Self::IcNodeCount => &["total_nodes", "up_nodes"],
            Self::IcSubnetTotal => &["ic_subnet_total"],
            Self::RegisteredCanistersCount => &["running_canisters", "stopped_canisters"],
            Self::TotalIcEnergyConsumptionRateKwh => &["energy_consumption_rate"],
            Self::BoundaryNodesCount => &["boundary_nodes_count"],
        }
    }
}

impl fmt::Display for IcMetricKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for IcMetricKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::all()
            .into_iter()
            .find(|metric| metric.as_str() == value)
            .ok_or_else(|| format!("unsupported IC Dashboard metric {value:?}"))
    }
}

///
/// IcMetricQuery
///
/// One explicitly bounded official Dashboard metric time-series query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcMetricQuery {
    /// Official Dashboard metric to retrieve.
    pub metric: IcMetricKind,
    /// Inclusive query start as Unix seconds.
    pub start_unix_secs: u64,
    /// Inclusive query end as Unix seconds.
    pub end_unix_secs: u64,
    /// Requested observation interval in seconds.
    pub step_secs: u32,
}

impl IcMetricQuery {
    /// Construct one explicit metric query window.
    #[must_use]
    pub const fn new(
        metric: IcMetricKind,
        start_unix_secs: u64,
        end_unix_secs: u64,
        step_secs: u32,
    ) -> Self {
        Self {
            metric,
            start_unix_secs,
            end_unix_secs,
            step_secs,
        }
    }
}

///
/// IcMetricRequest
///
/// Request accepted by the bounded official Dashboard metric report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcMetricRequest {
    /// Dashboard Metrics API base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Explicitly bounded metric query.
    pub query: IcMetricQuery,
}

impl IcMetricRequest {
    /// Construct one bounded live Dashboard metric request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        query: IcMetricQuery,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            query,
        }
    }
}

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

///
/// IcCanisterRequest
///
/// Request accepted by the official Dashboard canister report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterRequest {
    /// Dashboard API base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Canister principal to inspect.
    pub canister_id: String,
}

impl IcCanisterRequest {
    /// Construct a live Dashboard canister request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            canister_id: canister_id.into(),
        }
    }
}

///
/// IcCanisterFilters
///
/// Official Dashboard filters shared by canister count and page requests.
///

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct IcCanisterFilters {
    /// Select canisters according to whether the Dashboard records a name.
    pub has_name: Option<bool>,
    /// Select canisters assigned to this Subnet principal.
    pub subnet_id: Option<String>,
    /// Select canisters controlled by this principal.
    pub controller_id: Option<String>,
    /// Raw Dashboard language labels to include.
    pub languages: Vec<String>,
    /// Raw Dashboard canister classifications to include.
    pub canister_types: Vec<String>,
    /// Raw Dashboard text search, between two and one hundred characters.
    pub query: Option<String>,
}

///
/// IcCanisterCountRequest
///
/// Request for one bounded official Dashboard canister-count lookup.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterCountRequest {
    /// Dashboard API v4 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Filters applied by the Dashboard.
    pub filters: IcCanisterFilters,
}

impl IcCanisterCountRequest {
    /// Construct a live Dashboard canister-count request without filters.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            filters: IcCanisterFilters::default(),
        }
    }

    /// Set the Dashboard filters used by this request.
    #[must_use]
    pub fn with_filters(mut self, filters: IcCanisterFilters) -> Self {
        self.filters = filters;
        self
    }
}

///
/// IcCanisterPageRequest
///
/// Request for one bounded official Dashboard canister page.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterPageRequest {
    /// Dashboard API v4 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Filters applied by the Dashboard.
    pub filters: IcCanisterFilters,
    /// Maximum rows requested from the API.
    pub limit: u16,
    /// Exclusive forward cursor returned by an earlier page.
    pub after: Option<String>,
    /// Exclusive backward cursor returned by an earlier page.
    pub before: Option<String>,
}

impl IcCanisterPageRequest {
    /// Construct a live Dashboard page request with the default bounded limit.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            filters: IcCanisterFilters::default(),
            limit: crate::ic::DEFAULT_IC_CANISTER_PAGE_LIMIT,
            after: None,
            before: None,
        }
    }

    /// Set the Dashboard filters used by this request.
    #[must_use]
    pub fn with_filters(mut self, filters: IcCanisterFilters) -> Self {
        self.filters = filters;
        self
    }

    /// Set the maximum number of returned rows.
    #[must_use]
    pub const fn with_limit(mut self, limit: u16) -> Self {
        self.limit = limit;
        self
    }

    /// Set an exclusive forward cursor.
    #[must_use]
    pub fn with_after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    /// Set an exclusive backward cursor.
    #[must_use]
    pub fn with_before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }
}
