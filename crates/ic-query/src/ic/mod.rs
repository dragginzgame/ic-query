//! Official IC Dashboard API report models, adapters, builders, and renderers.

#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod live;
mod model;
#[cfg(feature = "host")]
mod source;
mod text;

#[cfg(feature = "host")]
pub use build::{
    build_ic_boundary_node_data_centers_report,
    build_ic_boundary_node_data_centers_report_with_source, build_ic_canister_count_report,
    build_ic_canister_count_report_with_source, build_ic_canister_page_report,
    build_ic_canister_page_report_with_source, build_ic_metric_report,
    build_ic_metric_report_with_source,
};
#[cfg(feature = "host")]
pub use build::{build_ic_canister_report, build_ic_canister_report_with_source};
#[cfg(feature = "host")]
pub use live::LiveIcSource;
pub use model::{
    IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersReport, IcBoundaryNodeDataCentersRequest,
    IcCanisterCountReport, IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageController,
    IcCanisterPageReport, IcCanisterPageRequest, IcCanisterPageRow, IcCanisterReport,
    IcCanisterRequest, IcCanisterUpgrade, IcDashboardReportProvenance, IcMetricKind,
    IcMetricObservation, IcMetricQuery, IcMetricReport, IcMetricRequest, IcMetricSeries,
};
#[cfg(feature = "host")]
pub use model::{
    IcBoundaryNodeDataCentersSourceData, IcCanisterCountSourceData, IcCanisterPageSourceData,
    IcCanisterSourceData, IcHostError, IcMetricSourceData, IcSourceRequest,
};
#[cfg(feature = "host")]
pub use source::{IcCanisterCollectionSource, IcCanisterSource, IcMetricSource, IcNetworkSource};
pub use text::{
    ic_boundary_node_data_centers_report_text, ic_canister_count_report_text,
    ic_canister_page_report_text, ic_canister_report_text, ic_metric_report_text,
};

/// Default base endpoint for the official IC Dashboard API.
pub const DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT: &str = "https://ic-api.internetcomputer.org/api/v3";

/// Default base endpoint for official IC Dashboard canister collection queries.
pub const DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT: &str =
    "https://ic-api.internetcomputer.org/api/v4";

/// Default base endpoint for the official IC Dashboard Metrics API.
pub const DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT: &str =
    "https://metrics-api.internetcomputer.org/api/v1";

/// Default base endpoint for official boundary-node data-center queries.
pub const DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT: &str =
    "https://ic-api.internetcomputer.org/api/v4";

/// Default row limit for one official Dashboard canister page.
pub const DEFAULT_IC_CANISTER_PAGE_LIMIT: u16 = 50;

/// Maximum row limit accepted for one official Dashboard canister page.
pub const MAX_IC_CANISTER_PAGE_LIMIT: u16 = 100;

/// Default relative window for one Dashboard metric query.
pub const DEFAULT_IC_METRIC_WINDOW_SECS: u64 = 3_600;

/// Default interval between requested Dashboard metric observations.
pub const DEFAULT_IC_METRIC_STEP_SECS: u32 = 300;

/// Earliest timestamp accepted by the official Dashboard Metrics API.
pub const MIN_IC_METRIC_TIMESTAMP: u64 = 1_620_432_000;

/// Largest step accepted by the official Dashboard Metrics API.
pub const MAX_IC_METRIC_STEP_SECS: u32 = 259_200;

/// Maximum requested observations accepted per metric series.
pub const MAX_IC_METRIC_OBSERVATIONS_PER_SERIES: u64 = 1_000;

/// Maximum boundary-node data-center rows accepted from one source response.
pub const MAX_IC_BOUNDARY_NODE_DATA_CENTERS: usize = 1_000;

#[cfg(feature = "host")]
const IC_DASHBOARD_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const IC_DASHBOARD_AUTHORITY: &str = "official_ic_dashboard_api";
#[cfg(feature = "host")]
const IC_DASHBOARD_NETWORK: &str = "ic";

#[cfg(all(test, feature = "host"))]
mod tests;
