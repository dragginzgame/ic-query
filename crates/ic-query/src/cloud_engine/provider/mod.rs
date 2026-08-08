//! Module: cloud_engine::provider
//!
//! Responsibility: official Dashboard CloudEngine provider requests, reports, validation, and text.
//! Does not own: native control-plane queries, Registry inventory, CLI parsing, or caching.
//! Boundary: filters one complete Dashboard provider resource by explicit CloudEngine evidence.

#[cfg(feature = "dashboard-host")]
mod host;
mod model;
mod text;

#[cfg(feature = "dashboard-host")]
pub use host::{
    CloudEngineProviderSource, build_cloud_engine_provider_info_report,
    build_cloud_engine_provider_info_report_with_source, build_cloud_engine_provider_list_report,
    build_cloud_engine_provider_list_report_with_source,
};
pub use model::{
    CloudEngineProviderInfoReport, CloudEngineProviderInfoRequest, CloudEngineProviderListReport,
    CloudEngineProviderListRequest, CloudEngineProviderLocation, CloudEngineProviderRow,
};
#[cfg(feature = "dashboard-host")]
pub use model::{CloudEngineProviderInfoSourceData, CloudEngineProviderListSourceData};
pub use text::{cloud_engine_provider_info_report_text, cloud_engine_provider_list_report_text};

/// Default official Dashboard endpoint for CloudEngine provider reports.
pub const DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT: &str =
    "https://ic-api.internetcomputer.org/api/v3";

/// Maximum provider rows accepted from the complete Dashboard resource.
pub const MAX_CLOUD_ENGINE_PROVIDER_SOURCE_ROWS: usize = 1_000;

/// Maximum general or CloudEngine locations accepted for one provider.
pub const MAX_CLOUD_ENGINE_PROVIDER_LOCATIONS: usize = 100;

#[cfg(test)]
mod tests;
