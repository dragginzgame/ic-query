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
    build_ic_canister_count_report, build_ic_canister_count_report_with_source,
    build_ic_canister_page_report, build_ic_canister_page_report_with_source,
};
#[cfg(feature = "host")]
pub use build::{build_ic_canister_report, build_ic_canister_report_with_source};
#[cfg(feature = "host")]
pub use live::LiveIcSource;
pub use model::{
    IcCanisterCountReport, IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageController,
    IcCanisterPageReport, IcCanisterPageRequest, IcCanisterPageRow, IcCanisterReport,
    IcCanisterRequest, IcCanisterUpgrade, IcDashboardReportProvenance,
};
#[cfg(feature = "host")]
pub use model::{
    IcCanisterCountSourceData, IcCanisterPageSourceData, IcCanisterSourceData, IcHostError,
    IcSourceRequest,
};
#[cfg(feature = "host")]
pub use source::{IcCanisterCollectionSource, IcCanisterSource};
pub use text::{
    ic_canister_count_report_text, ic_canister_page_report_text, ic_canister_report_text,
};

/// Default base endpoint for the official IC Dashboard API.
pub const DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT: &str = "https://ic-api.internetcomputer.org/api/v3";

/// Default base endpoint for official IC Dashboard canister collection queries.
pub const DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT: &str =
    "https://ic-api.internetcomputer.org/api/v4";

/// Default row limit for one official Dashboard canister page.
pub const DEFAULT_IC_CANISTER_PAGE_LIMIT: u16 = 50;

/// Maximum row limit accepted for one official Dashboard canister page.
pub const MAX_IC_CANISTER_PAGE_LIMIT: u16 = 100;

#[cfg(feature = "host")]
const IC_CANISTER_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const IC_DASHBOARD_AUTHORITY: &str = "official_ic_dashboard_api";
#[cfg(feature = "host")]
const IC_DASHBOARD_NETWORK: &str = "ic";

#[cfg(all(test, feature = "host"))]
mod tests;
