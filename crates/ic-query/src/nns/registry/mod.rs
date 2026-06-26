pub mod report;

#[cfg(feature = "cli")]
mod commands;
#[cfg(feature = "cli")]
mod options;
#[cfg(feature = "cli")]
mod run;

pub use report::{
    DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT, NnsRegistryVersionReport, NnsRegistryVersionRequest,
    nns_registry_version_report_text,
};
#[cfg(feature = "host")]
pub use report::{NnsRegistryHostError, build_nns_registry_version_report};

#[cfg(all(test, feature = "cli"))]
pub(super) use commands::{registry_usage, registry_version_usage};
#[cfg(all(test, feature = "cli"))]
pub(super) use options::RegistryVersionOptions;
#[cfg(feature = "cli")]
pub(super) use run::run;
