//! Module: ic::model
//!
//! Responsibility: public IC Dashboard requests, source data, reports, and errors.
//! Does not own: HTTP transport, source validation, report assembly, or rendering.
//! Boundary: preserves raw Dashboard values and explicit off-chain provenance.

#[cfg(feature = "host")]
use crate::runtime::RuntimeError;
use serde::Serialize;
use std::{fmt, str::FromStr};
#[cfg(feature = "host")]
use thiserror::Error as ThisError;

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
            limit: super::DEFAULT_IC_CANISTER_PAGE_LIMIT,
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

///
/// IcCanisterUpgrade
///
/// One proposal-linked canister upgrade recorded by the Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterUpgrade {
    /// Proposal execution time as raw Unix seconds.
    pub executed_timestamp_seconds: u64,
    /// Wasm module hash as raw lowercase hexadecimal text.
    pub module_hash: String,
    /// NNS proposal that installed this module.
    pub proposal_id: u64,
}

///
/// IcDashboardReportProvenance
///
/// Shared off-chain provenance and authority guarantees for Dashboard reports.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcDashboardReportProvenance {
    /// Report schema version.
    pub schema_version: u32,
    /// Network represented by the official Dashboard API.
    pub network: String,
    /// Authority that supplied the report fields.
    pub authority: String,
    /// Dashboard API base endpoint queried by the source.
    pub source_endpoint: String,
    /// Time this report was collected.
    pub fetched_at: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Whether the API response is cryptographically certified IC state.
    pub certified: bool,
    /// Whether every returned value is guaranteed to describe one point in time.
    pub point_in_time_guaranteed: bool,
}

///
/// IcMetricObservation
///
/// One raw timestamp and value returned by the Dashboard Metrics API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcMetricObservation {
    /// Observation timestamp as Unix seconds.
    pub timestamp_unix_secs: u64,
    /// Raw value string returned by the Dashboard.
    pub value: String,
}

///
/// IcMetricSeries
///
/// One named raw series in a Dashboard metric response.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcMetricSeries {
    /// Raw Dashboard response field that names this series.
    pub name: String,
    /// Observations in strictly increasing timestamp order.
    pub observations: Vec<IcMetricObservation>,
}

///
/// IcMetricReport
///
/// One bounded time-series response from the official Dashboard Metrics API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcMetricReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Metric and explicit time-series bounds, flattened in report JSON.
    #[serde(flatten)]
    pub query: IcMetricQuery,
    /// Number of named series returned by the API.
    pub returned_series_count: usize,
    /// Total number of observations across all returned series.
    pub returned_observation_count: usize,
    /// Raw named time series in canonical series-name order.
    pub series: Vec<IcMetricSeries>,
}

///
/// IcCanisterReport
///
/// One live canister metadata report from the official Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Canonical canister principal.
    pub canister_id: String,
    /// Dashboard database row identifier.
    pub dashboard_id: u64,
    /// Raw optional Dashboard canister classification.
    pub canister_type: Option<String>,
    /// Raw Dashboard canister name; an empty string means no name was recorded.
    pub name: String,
    /// Canonical Subnet principal recorded by the Dashboard.
    pub subnet_id: String,
    /// Canonically ordered controller principals recorded by the Dashboard.
    pub controllers: Vec<String>,
    /// Raw Dashboard language label; an empty string means no language was recorded.
    pub language: String,
    /// Raw current module hash; an empty string means no hash was recorded.
    pub module_hash: String,
    /// Raw Dashboard row update timestamp.
    pub dashboard_updated_at: String,
    /// Number of proposal-linked upgrades when history is available.
    pub upgrade_count: Option<usize>,
    /// Proposal-linked upgrade history, or `None` when the Dashboard returned `null`.
    pub upgrades: Option<Vec<IcCanisterUpgrade>>,
}

///
/// IcCanisterCountReport
///
/// One filtered canister count from the official Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterCountReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Filters applied by the Dashboard.
    pub filters: IcCanisterFilters,
    /// Number of matching Dashboard canister records.
    pub total: u64,
}

///
/// IcCanisterPageController
///
/// One controller entry returned by the Dashboard canister collection API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterPageController {
    /// Canonical controller principal.
    pub principal_id: String,
    /// Raw optional Dashboard metadata associated with the controller.
    pub raw_metadata: Option<String>,
}

///
/// IcCanisterPageRow
///
/// One discovery row from a bounded Dashboard canister page.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterPageRow {
    /// Canonical canister principal.
    pub canister_id: String,
    /// Dashboard database row identifier.
    pub dashboard_id: u64,
    /// Raw optional Dashboard canister classification.
    pub canister_type: Option<String>,
    /// Raw Dashboard canister name.
    pub name: String,
    /// Canonical Subnet principal recorded by the Dashboard.
    pub subnet_id: String,
    /// Canonically ordered controller entries recorded by the Dashboard.
    pub controllers: Vec<IcCanisterPageController>,
    /// Raw Dashboard language label.
    pub language: String,
    /// Raw current module hash.
    pub module_hash: String,
    /// Raw Dashboard row update timestamp.
    pub dashboard_updated_at: String,
}

///
/// IcCanisterPageReport
///
/// One explicitly bounded page from the official Dashboard canister collection.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterPageReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Filters applied by the Dashboard.
    pub filters: IcCanisterFilters,
    /// Maximum rows requested from the API.
    pub requested_limit: u16,
    /// Number of rows returned in this report.
    pub returned_count: usize,
    /// Exclusive forward cursor supplied to this request.
    pub after: Option<String>,
    /// Exclusive backward cursor supplied to this request.
    pub before: Option<String>,
    /// Cursor for an explicit request for the preceding page.
    pub previous_cursor: Option<String>,
    /// Cursor for an explicit request for the following page.
    pub next_cursor: Option<String>,
    /// Canister discovery rows in Dashboard canister-id order.
    pub rows: Vec<IcCanisterPageRow>,
}

///
/// IcSourceRequest
///
/// Shared endpoint and collection provenance for IC Dashboard source calls and results.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcSourceRequest {
    /// Dashboard API base endpoint.
    pub endpoint: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// Collector identity recorded in report provenance.
    pub fetched_by: String,
}

#[cfg(feature = "host")]
impl IcSourceRequest {
    /// Construct source-call provenance.
    #[must_use]
    pub fn new(
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

///
/// IcCanisterSourceData
///
/// Raw canister metadata and provenance returned by an IC Dashboard source.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Canister principal returned by the Dashboard.
    pub canister_id: String,
    /// Dashboard database row identifier.
    pub dashboard_id: u64,
    /// Raw optional Dashboard canister classification.
    pub canister_type: Option<String>,
    /// Raw Dashboard canister name.
    pub name: String,
    /// Subnet principal returned by the Dashboard.
    pub subnet_id: String,
    /// Controller principals returned by the Dashboard.
    pub controllers: Vec<String>,
    /// Raw Dashboard language label.
    pub language: String,
    /// Raw current module hash.
    pub module_hash: String,
    /// Raw Dashboard row update timestamp.
    pub dashboard_updated_at: String,
    /// Proposal-linked upgrades, or `None` when the Dashboard returned `null`.
    pub upgrades: Option<Vec<IcCanisterUpgrade>>,
}

///
/// IcCanisterCountSourceData
///
/// Raw filtered count and provenance returned by a Dashboard source.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterCountSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Filters applied by the source.
    pub filters: IcCanisterFilters,
    /// Number of matching Dashboard canister records.
    pub total: u64,
}

///
/// IcCanisterPageSourceData
///
/// Raw bounded canister page and provenance returned by a Dashboard source.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterPageSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Filters applied by the source.
    pub filters: IcCanisterFilters,
    /// Maximum rows requested from the source.
    pub requested_limit: u16,
    /// Exclusive forward cursor supplied to the source.
    pub after: Option<String>,
    /// Exclusive backward cursor supplied to the source.
    pub before: Option<String>,
    /// Cursor for an explicit request for the preceding page.
    pub previous_cursor: Option<String>,
    /// Cursor for an explicit request for the following page.
    pub next_cursor: Option<String>,
    /// Canister discovery rows returned by the source.
    pub rows: Vec<IcCanisterPageRow>,
}

///
/// IcMetricSourceData
///
/// Raw bounded metric series and provenance returned by a Dashboard source.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcMetricSourceData {
    /// Source request and provenance preserved by the source.
    pub source: IcSourceRequest,
    /// Metric query applied by the source.
    pub query: IcMetricQuery,
    /// Raw named time series returned by the source.
    pub series: Vec<IcMetricSeries>,
}

///
/// IcHostError
///
/// Typed error returned by IC Dashboard report builders and live sources.
///

#[cfg(feature = "host")]
#[derive(Debug, ThisError)]
pub enum IcHostError {
    /// The synchronous adapter could not create its local async runtime.
    #[error("failed to run IC Dashboard query: {0}")]
    Runtime(#[from] RuntimeError),

    /// A request supplied an invalid canister principal.
    #[error("invalid {field}: {reason}")]
    InvalidPrincipal {
        /// Principal field being validated.
        field: &'static str,
        /// Principal parser diagnostic.
        reason: String,
    },

    /// A request violates the bounded Dashboard query contract.
    #[error("invalid {field}: {reason}")]
    InvalidRequest {
        /// Request field being validated.
        field: &'static str,
        /// Deterministic validation diagnostic.
        reason: String,
    },

    /// The Dashboard API base endpoint is malformed or unsupported.
    #[error("invalid IC Dashboard endpoint {endpoint}: {reason}")]
    InvalidEndpoint {
        /// Rejected endpoint.
        endpoint: String,
        /// URL validation diagnostic.
        reason: String,
    },

    /// The HTTP client could not be constructed.
    #[error("failed to build IC Dashboard HTTP client: {reason}")]
    HttpClientBuild {
        /// HTTP client construction diagnostic.
        reason: String,
    },

    /// The live Dashboard request failed before a response was received.
    #[error("IC Dashboard request to {url} failed: {reason}")]
    HttpRequest {
        /// Fully resolved request URL.
        url: String,
        /// HTTP transport error.
        reason: String,
    },

    /// The Dashboard returned a non-success HTTP status.
    #[error("IC Dashboard request to {url} returned HTTP status {status}")]
    HttpStatus {
        /// Fully resolved request URL.
        url: String,
        /// Numeric HTTP status.
        status: u16,
    },

    /// The Dashboard response did not match the expected JSON shape.
    #[error("failed to decode IC Dashboard response from {url}: {reason}")]
    JsonDecode {
        /// Fully resolved request URL.
        url: String,
        /// JSON response decoding error.
        reason: String,
    },

    /// A source capability returned data that violates its public result contract.
    #[error("invalid IC Dashboard source data: {reason}")]
    InvalidSourceData {
        /// Deterministic invariant failure.
        reason: String,
    },
}
