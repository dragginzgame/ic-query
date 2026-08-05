//! NNS registry-version requests, reports, host builders, and renderers.

mod report;

pub use report::{
    DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT, NnsCertifiedRegistryDeltaBatchReport,
    NnsCertifiedRegistryDeltaBatchRequest, NnsCertifiedRegistryDeltaLimits,
    NnsCertifiedRegistryDeltaVersion, NnsCertifiedRegistryMutation,
    NnsCertifiedRegistryMutationKind, NnsCertifiedRegistryPrecondition, NnsRegistryCertification,
    NnsRegistryVersionReport, NnsRegistryVersionRequest, nns_registry_version_report_text,
};
#[cfg(feature = "nns-host")]
pub use report::{
    NnsCertifiedRegistryDeltaSource, NnsCertifiedRegistryDeltaSourceFuture, NnsRegistryHostError,
    NnsRegistrySource, NnsRegistryVersionData, build_nns_registry_version_report,
    build_nns_registry_version_report_with_source, fetch_nns_certified_registry_delta_batch_async,
    fetch_nns_certified_registry_delta_batch_with_source_async,
    nns_certified_registry_delta_limits, validate_nns_certified_registry_delta_batch,
};
