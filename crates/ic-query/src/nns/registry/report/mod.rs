#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod error;
mod model;
#[cfg(feature = "nns-host")]
mod source;
mod text;

#[cfg(feature = "nns-host")]
pub use build::{build_nns_registry_version_report, build_nns_registry_version_report_with_source};
#[cfg(feature = "nns-host")]
pub use error::NnsRegistryHostError;
pub use model::{NnsRegistryVersionReport, NnsRegistryVersionRequest};
#[cfg(feature = "nns-host")]
pub use source::{NnsRegistrySource, NnsRegistryVersionData};
pub use text::nns_registry_version_report_text;

pub const DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[cfg(all(test, feature = "nns-host"))]
mod tests;
