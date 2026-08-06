//! NNS registry-version requests, reports, host builders, and renderers.

#[cfg(feature = "nns-host")]
mod replay;
mod report;

#[cfg(feature = "nns-host")]
pub use replay::{
    NNS_CERTIFIED_REGISTRY_ARCHIVE_MANIFEST_SCHEMA_VERSION,
    NNS_REGISTRY_REPLAY_PROVENANCE_SCHEMA_VERSION, NnsAuthenticatedRegistryArchive,
    NnsAuthenticatedRegistryReplayBuilder, NnsAuthenticatedRegistryReplaySession,
    NnsCertifiedRegistryArchiveBatchDescriptor, NnsCertifiedRegistryArchiveBootstrapError,
    NnsCertifiedRegistryArchiveBootstrapRequest, NnsCertifiedRegistryArchiveError,
    NnsCertifiedRegistryArchiveLimits, NnsCertifiedRegistryArchiveManifest,
    NnsCertifiedRegistryArchiveManifestBuilder, NnsCertifiedRegistryArchivePublisher,
    NnsCertifiedRegistryArchiveStorageError, NnsCertifiedRegistryArchiveStorageLimits,
    NnsCertifiedRegistryBootstrapProbeOutcome, NnsCertifiedRegistryBootstrapProbeStatus,
    NnsCertifiedRegistryBootstrapRequest, NnsCertifiedSubnetCatalogAuthority,
    NnsCertifiedSubnetCatalogFreshness, NnsCertifiedSubnetCatalogProjectionRequest,
    NnsCertifiedSubnetCatalogVersionPolicy, NnsRegistryReplayError, NnsRegistryReplayLimits,
    NnsRegistryReplayProgress, NnsRegistryReplaySession, NnsRegistryReplaySessionLimits,
    NnsRegistryReplayState, NnsRegistryReplayValue, NnsRegistrySubnetCatalogProjection,
    NnsRegistrySubnetCatalogProjectionError, apply_nns_certified_registry_delta_batch,
    bootstrap_nns_certified_registry_archive_async,
    bootstrap_nns_certified_registry_archive_with_source_async,
    bootstrap_nns_certified_registry_async, bootstrap_nns_certified_registry_with_source_async,
    load_nns_certified_registry_archive, nns_certified_registry_archive_manifest_path,
    nns_certified_registry_archive_refresh_lock_path, probe_nns_certified_registry_async,
    probe_nns_certified_registry_with_source_async, project_nns_certified_subnet_catalog,
    project_nns_registry_subnet_catalog, validate_nns_certified_registry_archive_manifest,
};
pub use report::{
    DEFAULT_NNS_REGISTRY_SOURCE_ENDPOINT, NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION,
    NnsCertifiedRegistryChunkEvidence, NnsCertifiedRegistryDeltaBatchReport,
    NnsCertifiedRegistryDeltaBatchRequest, NnsCertifiedRegistryDeltaLimits,
    NnsCertifiedRegistryDeltaVersion, NnsCertifiedRegistryMutation,
    NnsCertifiedRegistryMutationKind, NnsCertifiedRegistryPrecondition,
    NnsCertifiedRegistryValueEncoding, NnsRegistryCertification, NnsRegistryVersionReport,
    NnsRegistryVersionRequest, nns_registry_version_report_text,
};
#[cfg(feature = "nns-host")]
pub use report::{
    NnsAuthenticatedRegistryDeltaBatch, NnsCertifiedRegistryDeltaSource,
    NnsCertifiedRegistryDeltaSourceFuture, NnsRegistryHostError, NnsRegistrySource,
    NnsRegistryVersionData, build_nns_registry_version_report,
    build_nns_registry_version_report_with_source, fetch_nns_certified_registry_delta_batch_async,
    fetch_nns_certified_registry_delta_batch_with_source_async,
    nns_certified_registry_delta_limits, reauthenticate_nns_certified_registry_delta_batch,
    validate_nns_certified_registry_delta_batch,
};
