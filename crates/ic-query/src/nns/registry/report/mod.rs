#[cfg(feature = "nns-host")]
mod authentication;
#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
mod delta;
#[cfg(feature = "nns-host")]
mod error;
mod model;
#[cfg(feature = "nns-host")]
mod source;
mod text;

#[cfg(feature = "nns-host")]
pub use authentication::{
    NnsAuthenticatedRegistryDeltaBatch, reauthenticate_nns_certified_registry_delta_batch,
};
#[cfg(feature = "nns-host")]
pub use build::{build_nns_registry_version_report, build_nns_registry_version_report_with_source};
#[cfg(feature = "nns-host")]
pub use delta::{
    fetch_nns_certified_registry_delta_batch_async,
    fetch_nns_certified_registry_delta_batch_with_source_async,
    validate_nns_certified_registry_delta_batch,
};
#[cfg(feature = "nns-host")]
pub use error::NnsRegistryHostError;
pub use model::{
    NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION, NnsCertifiedRegistryChunkEvidence,
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    NnsCertifiedRegistryDeltaLimits, NnsCertifiedRegistryDeltaVersion,
    NnsCertifiedRegistryMutation, NnsCertifiedRegistryMutationKind,
    NnsCertifiedRegistryPrecondition, NnsCertifiedRegistryValueEncoding, NnsRegistryCertification,
    NnsRegistryVersionReport, NnsRegistryVersionRequest,
};
#[cfg(feature = "nns-host")]
pub use source::{
    NnsCertifiedRegistryDeltaSource, NnsCertifiedRegistryDeltaSourceFuture, NnsRegistrySource,
    NnsRegistryVersionData, nns_certified_registry_delta_limits,
};
pub use text::nns_registry_version_report_text;

pub const DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[cfg(all(test, feature = "nns-host"))]
mod tests;
