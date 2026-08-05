//! NNS registry-version requests, reports, host builders, and renderers.

mod report;

pub use report::{
    DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT, NnsRegistryVersionReport, NnsRegistryVersionRequest,
    nns_registry_version_report_text,
};
#[cfg(feature = "nns-host")]
pub use report::{
    NnsRegistryHostError, NnsRegistrySource, NnsRegistryVersionData,
    build_nns_registry_version_report, build_nns_registry_version_report_with_source,
};
