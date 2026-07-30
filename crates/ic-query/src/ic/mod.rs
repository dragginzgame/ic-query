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
pub use build::{build_ic_canister_report, build_ic_canister_report_with_source};
#[cfg(feature = "host")]
pub use live::LiveIcSource;
pub use model::{IcCanisterReport, IcCanisterRequest, IcCanisterUpgrade};
#[cfg(feature = "host")]
pub use model::{IcCanisterSourceData, IcHostError, IcSourceRequest};
#[cfg(feature = "host")]
pub use source::IcCanisterSource;
pub use text::ic_canister_report_text;

/// Default base endpoint for the official IC Dashboard API.
pub const DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT: &str = "https://ic-api.internetcomputer.org/api/v3";

#[cfg(feature = "host")]
const IC_CANISTER_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "host")]
const IC_DASHBOARD_AUTHORITY: &str = "official_ic_dashboard_api";
#[cfg(feature = "host")]
const IC_DASHBOARD_NETWORK: &str = "ic";

#[cfg(all(test, feature = "host"))]
mod tests;
