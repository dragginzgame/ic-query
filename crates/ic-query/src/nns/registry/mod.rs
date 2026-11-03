//! NNS registry-version requests, reports, host builders.

mod report;

pub use report::{
    DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT, NnsRegistryCertification, NnsRegistryInventoryProvenance,
    NnsRegistryVersionData, NnsRegistryVersionReport, NnsRegistryVersionRequest,
};
#[cfg(feature = "nns-host")]
pub use report::{
    NnsRegistryHostError, NnsRegistrySource, build_nns_registry_version_report,
    build_nns_registry_version_report_with_source,
};
