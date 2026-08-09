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
    IcCanisterSourceData, IcDailyStatsSourceData, IcIcrcAccountInfoSourceData,
    IcIcrcAccountListSourceData, IcIcrcAccountSourceRow, IcIcrcHolderListSourceData,
    IcIcrcHolderSourceRow, IcIcrcIndexedCountSourceData, IcIcrcTokenValueSourceData,
    IcIcrcTokenValueSourceRow, IcIcrcTotalSupplySourceData, IcMetricSourceData,
    IcNodeProviderRewardHistorySourceData, IcNodeProviderRewardInfoSourceData,
    IcNodeProviderRewardListSourceData, IcReplicaVersionInfoSourceData,
    IcReplicaVersionListSourceData, IcSourceRequest,
};
#[cfg(feature = "dashboard-host")]
pub use error::IcHostError;
pub use reports::{
    IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersReport, IcCanisterCountReport,
    IcCanisterPageController, IcCanisterPageReport, IcCanisterPageRow, IcCanisterReport,
    IcCanisterUpgrade, IcDailyStatsReport, IcDailyStatsRow, IcDashboardReportProvenance,
    IcIcrcAccountInfoReport, IcIcrcAccountListReport, IcIcrcAccountRow, IcIcrcHolderListReport,
    IcIcrcHolderRow, IcIcrcIndexedCountReport, IcIcrcTokenValueReport, IcIcrcTokenValueRow,
    IcIcrcTotalSupplyObservation, IcIcrcTotalSupplyReport, IcMetricObservation, IcMetricReport,
    IcMetricSeries, IcNodeProviderRewardHistoryObservation, IcNodeProviderRewardHistoryReport,
    IcNodeProviderRewardInfoReport, IcNodeProviderRewardListReport, IcNodeProviderRewardRow,
    IcNodeProviderRewardXdrConversionRate, IcReplicaVersionInfoReport, IcReplicaVersionListReport,
    IcReplicaVersionListRow, IcReplicaVersionStatus, IcReplicaVersionSubnetRollout,
};
pub use requests::{
    IcBoundaryNodeDataCentersRequest, IcCanisterCountRequest, IcCanisterFilters,
    IcCanisterPageRequest, IcCanisterRequest, IcDailyStatsQuery, IcDailyStatsRequest,
    IcIcrcAccountInfoRequest, IcIcrcAccountListQuery, IcIcrcAccountListRequest, IcIcrcAccountSort,
    IcIcrcAnalyticsRequest, IcIcrcHolderListQuery, IcIcrcHolderListRequest, IcIcrcHolderSort,
    IcIcrcIndexedCountKind, IcIcrcIndexedCountRequest, IcIcrcTokenValueQuery,
    IcIcrcTokenValueRequest, IcIcrcTotalSupplyQuery, IcIcrcTotalSupplyRequest, IcMetricKind,
    IcMetricQuery, IcMetricRequest, IcNodeProviderRewardHistoryQuery,
    IcNodeProviderRewardHistoryRequest, IcNodeProviderRewardInfoRequest,
    IcNodeProviderRewardListQuery, IcNodeProviderRewardListRequest, IcReplicaVersionInfoRequest,
    IcReplicaVersionListQuery, IcReplicaVersionListRequest,
};
