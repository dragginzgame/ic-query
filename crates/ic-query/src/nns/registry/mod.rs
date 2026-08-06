//! NNS registry-version requests, reports, host builders, and renderers.

#[cfg(feature = "nns-host")]
mod replay;
mod report;

#[cfg(feature = "nns-host")]
pub use replay::{
    NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION, NnsAuthenticatedRegistryReplaySession,
    NnsAuthenticatedRegistrySubnetCatalogProjection, NnsCertifiedRegistryBootstrapProbeOutcome,
    NnsCertifiedRegistryBootstrapProbeStatus, NnsCertifiedRegistryBootstrapRequest,
    NnsRegistryReplayError, NnsRegistryReplayLimits, NnsRegistryReplayProgress,
    NnsRegistryReplaySession, NnsRegistryReplaySessionLimits, NnsRegistryReplayState,
    NnsRegistryReplayValue, NnsRegistrySubnetCatalogProjection,
    NnsRegistrySubnetCatalogProjectionError, apply_nns_certified_registry_delta_batch,
    bootstrap_nns_certified_registry_async, bootstrap_nns_certified_registry_with_source_async,
    probe_nns_certified_registry_async, probe_nns_certified_registry_with_source_async,
    project_nns_authenticated_registry_subnet_catalog, project_nns_registry_subnet_catalog,
};
pub use report::{
    DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT, NnsCertifiedRegistryChunkEvidence,
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    NnsCertifiedRegistryDeltaLimits, NnsCertifiedRegistryDeltaVersion,
    NnsCertifiedRegistryMutation, NnsCertifiedRegistryMutationKind,
    NnsCertifiedRegistryPrecondition, NnsCertifiedRegistryValueEncoding, NnsRegistryCertification,
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
