#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
mod error;
mod model;
#[cfg(feature = "host")]
mod source;
mod text;

#[cfg(feature = "host")]
pub use build::{build_nns_registry_version_report, build_nns_registry_version_report_with_source};
#[cfg(feature = "host")]
pub use error::NnsRegistryHostError;
pub use model::{NnsRegistryVersionReport, NnsRegistryVersionRequest};
#[cfg(feature = "host")]
pub use source::{
    LiveNnsRegistrySource, NnsRegistrySource, NnsRegistrySourceRequest, NnsRegistryVersionData,
};
pub use text::nns_registry_version_report_text;

pub const DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[cfg(all(test, feature = "host"))]
mod tests;
