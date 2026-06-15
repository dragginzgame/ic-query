mod build;
mod error;
mod model;
mod source;
mod text;

pub use build::build_nns_registry_version_report;
pub use error::NnsRegistryHostError;
pub use model::NnsRegistryVersionRequest;
pub use text::nns_registry_version_report_text;

use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;

#[cfg(test)]
use crate::ic_registry::{MainnetRegistryFetchRequest, MainnetRegistryVersion};
#[cfg(test)]
use build::build_nns_registry_version_report_with_source;
#[cfg(test)]
use model::NnsRegistryVersionReport;
#[cfg(test)]
use source::NnsRegistrySource;

pub const DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;

#[cfg(test)]
mod tests;
