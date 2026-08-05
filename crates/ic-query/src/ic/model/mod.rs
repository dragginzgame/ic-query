//! Module: ic::model
//!
//! Responsibility: expose typed Dashboard requests, reports, source data, and errors.
//! Does not own: HTTP transport, source validation, report assembly, or rendering.
//! Boundary: keeps caller intent, serialized evidence, and host contracts separate.

#[cfg(feature = "dashboard-host")]
mod data;
#[cfg(feature = "dashboard-host")]
mod error;
mod reports;
mod requests;

#[cfg(feature = "dashboard-host")]
pub use data::{
    IcBoundaryNodeDataCentersSourceData, IcCanisterCountSourceData, IcCanisterPageSourceData,
    IcCanisterSourceData, IcDailyStatsSourceData, IcIcrcIndexedCountSourceData,
    IcIcrcTokenValueSourceData, IcIcrcTokenValueSourceRow, IcIcrcTotalSupplySourceData,
    IcMetricSourceData, IcSourceRequest,
};
#[cfg(feature = "dashboard-host")]
pub use error::IcHostError;
pub use reports::{
    IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersReport, IcCanisterCountReport,
    IcCanisterPageController, IcCanisterPageReport, IcCanisterPageRow, IcCanisterReport,
    IcCanisterUpgrade, IcDailyStatsReport, IcDailyStatsRow, IcDashboardReportProvenance,
    IcIcrcIndexedCountReport, IcIcrcTokenValueReport, IcIcrcTokenValueRow,
    IcIcrcTotalSupplyObservation, IcIcrcTotalSupplyReport, IcMetricObservation, IcMetricReport,
    IcMetricSeries,
};
pub use requests::{
    IcBoundaryNodeDataCentersRequest, IcCanisterCountRequest, IcCanisterFilters,
    IcCanisterPageRequest, IcCanisterRequest, IcDailyStatsQuery, IcDailyStatsRequest,
    IcIcrcAnalyticsRequest, IcIcrcIndexedCountKind, IcIcrcIndexedCountRequest,
    IcIcrcTokenValueQuery, IcIcrcTokenValueRequest, IcIcrcTotalSupplyQuery,
    IcIcrcTotalSupplyRequest, IcMetricKind, IcMetricQuery, IcMetricRequest,
};
