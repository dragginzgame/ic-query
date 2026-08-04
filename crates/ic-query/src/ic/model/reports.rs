//! Module: ic::model::reports
//!
//! Responsibility: public serialized Dashboard report, row, and provenance contracts.
//! Does not own: requests, host source data, errors, transport, or projection.
//! Boundary: preserves raw Dashboard values and explicit off-chain provenance.

use super::requests::{
    IcCanisterFilters, IcDailyStatsQuery, IcIcrcIndexedCountKind, IcIcrcTokenValueQuery,
    IcIcrcTotalSupplyQuery, IcMetricQuery,
};
use serde::Serialize;

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
/// IcIcrcTotalSupplyObservation
///
/// One raw ICRC ledger total-supply observation returned by the Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcTotalSupplyObservation {
    /// Observation timestamp as Unix seconds.
    pub timestamp_unix_secs: u64,
    /// Raw total supply in ledger base units.
    pub total_supply_base_units: String,
}

///
/// IcIcrcTotalSupplyReport
///
/// One bounded total-supply series from the official Dashboard ICRC API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcTotalSupplyReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Canonical ICRC ledger canister principal requested from the API.
    pub ledger_canister_id: String,
    /// Exact requested time bounds, flattened in report JSON.
    #[serde(flatten)]
    pub query: IcIcrcTotalSupplyQuery,
    /// Maximum observations implied by the requested inclusive window.
    pub requested_observation_limit: u64,
    /// Number of observations returned by the API.
    pub returned_observation_count: usize,
    /// Raw observations in strictly increasing timestamp order.
    pub observations: Vec<IcIcrcTotalSupplyObservation>,
}

///
/// IcIcrcIndexedCountReport
///
/// One current scalar count from the official Dashboard ICRC index.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcIndexedCountReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Canonical ICRC ledger canister principal requested from the API.
    pub ledger_canister_id: String,
    /// Indexed resource represented by this count.
    pub kind: IcIcrcIndexedCountKind,
    /// Number of matching resources currently represented by the Dashboard index.
    pub total: u64,
}

///
/// IcIcrcTokenValueRow
///
/// One raw externally sourced token-value record returned by the Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcTokenValueRow {
    /// Raw legacy price field in USD, when returned.
    pub price: Option<String>,
    /// Raw legacy 24-hour volume field in USD, when returned.
    pub volume_24h: Option<String>,
    /// Raw explicit price-in-USD field, when returned.
    pub price_usd: Option<String>,
    /// Raw explicit 24-hour volume-in-USD field, when returned.
    pub volume_24h_usd: Option<String>,
    /// External value provider named by the Dashboard, when returned.
    pub source: Option<String>,
    /// External value-provider URL returned by the Dashboard, when present.
    pub source_url: Option<String>,
    /// Observation timestamp as Unix seconds.
    pub timestamp_unix_secs: u64,
}

///
/// IcIcrcTokenValueReport
///
/// One bounded token-value series from the official Dashboard ICRC API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcTokenValueReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Canonical ICRC ledger canister principal requested from the API.
    pub ledger_canister_id: String,
    /// Exact requested time and row bounds, flattened in report JSON.
    #[serde(flatten)]
    pub query: IcIcrcTokenValueQuery,
    /// Number of rows returned by the API.
    pub returned_row_count: usize,
    /// Whether the response reached the requested limit and may be truncated.
    pub limit_reached: bool,
    /// Raw token-value rows in nondecreasing timestamp order.
    pub rows: Vec<IcIcrcTokenValueRow>,
}

///
/// IcDailyStatsRow
///
/// Selected raw daily network-activity values returned by the Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcDailyStatsRow {
    /// Raw UTC calendar day returned by the Dashboard.
    pub day: String,
    /// Observation timestamp as Unix seconds.
    pub timestamp_unix_secs: u64,
    /// Raw average query-transaction rate.
    pub average_query_transactions_per_second: String,
    /// Raw average update-transaction rate.
    pub average_update_transactions_per_second: String,
    /// Raw average total-transaction rate.
    pub average_transactions_per_second: String,
    /// Raw maximum query-transaction rate.
    pub max_query_transactions_per_second: String,
    /// Raw maximum update-transaction rate.
    pub max_update_transactions_per_second: String,
    /// Raw maximum total-transaction rate.
    pub max_total_transactions_per_second: String,
    /// Raw average block-production rate.
    pub blocks_per_second_average: String,
}

///
/// IcDailyStatsReport
///
/// One bounded daily network-activity response from the official Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcDailyStatsReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Exact requested time bounds, flattened in report JSON.
    #[serde(flatten)]
    pub query: IcDailyStatsQuery,
    /// Number of daily rows returned by the API.
    pub returned_day_count: usize,
    /// Rows in strictly increasing timestamp order.
    pub rows: Vec<IcDailyStatsRow>,
}

///
/// IcBoundaryNodeDataCenterRow
///
/// One raw data-center aggregate returned by the boundary-node API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcBoundaryNodeDataCenterRow {
    /// Dashboard data-center identifier.
    pub dc_id: String,
    /// Raw data-center display name.
    pub name: String,
    /// Raw infrastructure-owner label.
    pub owner: String,
    /// Raw Dashboard region label.
    pub region: String,
    /// Raw decimal latitude.
    pub latitude: String,
    /// Raw decimal longitude.
    pub longitude: String,
    /// Raw decimal count of boundary nodes assigned to this data center.
    pub total_nodes: String,
}

///
/// IcBoundaryNodeDataCentersReport
///
/// One complete response from the official boundary-node data-center resource.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcBoundaryNodeDataCentersReport {
    /// Shared Dashboard provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Number of data-center rows returned by the API.
    pub data_center_count: usize,
    /// Sum of the raw per-data-center boundary-node counts.
    pub total_node_count: u64,
    /// Rows in canonical data-center-id order, including zero-node locations.
    pub rows: Vec<IcBoundaryNodeDataCenterRow>,
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
