//! Module: nns::registry::replay::archive::bootstrap
//!
//! Responsibility: collect and atomically publish one bounded certified Registry archive.
//! Does not own: default paths, read-through policy, incremental refresh, cleanup, or CLI.
//! Boundary: every report is locally reauthenticated before durable archive admission.

use super::{
    NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchivePublisher,
    NnsCertifiedRegistryArchiveStorageError, NnsCertifiedRegistryArchiveStorageLimits,
    nns_certified_registry_archive_manifest_path,
    storage::{ArchiveBatchAuthenticator, BuiltInArchiveAuthenticator},
};
use crate::{
    cache_file::{
        CacheFileError, RefreshLockRequest, create_managed_parent_directory,
        with_refresh_lock_async,
    },
    nns::{
        LiveNnsSource,
        registry::{
            NnsCertifiedRegistryBootstrapRequest, NnsCertifiedRegistryDeltaBatchRequest,
            NnsCertifiedRegistryDeltaSource, NnsRegistryHostError, NnsRegistryReplayError,
            fetch_nns_certified_registry_delta_batch_with_source_async,
        },
    },
};
use std::path::{Path, PathBuf};
use thiserror::Error as ThisError;

const ARCHIVE_REFRESH_LOCK_FILE_NAME: &str = "refresh.lock";

///
/// NnsCertifiedRegistryArchiveBootstrapRequest
///
/// Explicit live collection, managed location, locking, and storage policy for one full archive.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryArchiveBootstrapRequest {
    /// Mainnet source, observation time, and cumulative replay ceilings.
    pub bootstrap: NnsCertifiedRegistryBootstrapRequest,
    /// Capability root that confines the archive and its refresh lock.
    pub cache_root: PathBuf,
    /// Caller-selected archive directory beneath `cache_root`.
    pub archive_root: PathBuf,
    /// Explicit manifest and retained-report storage ceilings.
    pub storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
    /// Age after which an abandoned archive refresh lock is reported as stale.
    pub lock_stale_after_seconds: u64,
}

impl NnsCertifiedRegistryArchiveBootstrapRequest {
    /// Create an explicit force-bootstrap request without selecting paths or limits by default.
    #[must_use]
    pub fn new(
        bootstrap: NnsCertifiedRegistryBootstrapRequest,
        cache_root: impl Into<PathBuf>,
        archive_root: impl Into<PathBuf>,
        storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            bootstrap,
            cache_root: cache_root.into(),
            archive_root: archive_root.into(),
            storage_limits,
            lock_stale_after_seconds,
        }
    }
}

///
/// NnsCertifiedRegistryArchiveBootstrapError
///
/// Typed replay, authentication, archive, and managed-filesystem failures from force bootstrap.
///

#[derive(Debug, ThisError)]
pub enum NnsCertifiedRegistryArchiveBootstrapError {
    /// Source evidence or bounded exact-target replay failed.
    #[error(transparent)]
    Replay(#[from] NnsRegistryReplayError),

    /// A structurally valid source report failed local mainnet evidence authentication.
    #[error(
        "certified Registry archive batch after version {requested_version} failed local authentication: {source}"
    )]
    BatchAuthentication {
        /// Registry version after which the rejected report requested changes.
        requested_version: u64,
        /// Typed certificate, witness, chunk, or committed-content failure.
        #[source]
        source: NnsRegistryHostError,
    },

    /// Confined archive publication or refresh-lock management failed.
    #[error(transparent)]
    Storage(#[from] NnsCertifiedRegistryArchiveStorageError),
}

/// Return the dedicated refresh-lock path for one caller-selected archive directory.
#[must_use]
pub fn nns_certified_registry_archive_refresh_lock_path(archive_root: &Path) -> PathBuf {
    archive_root.join(ARCHIVE_REFRESH_LOCK_FILE_NAME)
}

/// Force-bootstrap and atomically publish one complete archive from the live mainnet source.
///
/// This explicit operation begins at Registry version zero and may make multiple bounded source
/// calls. It never runs from an archive load or ordinary Subnet Catalog read-through policy.
pub async fn bootstrap_nns_certified_registry_archive_async(
    request: &NnsCertifiedRegistryArchiveBootstrapRequest,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveBootstrapError> {
    bootstrap_nns_certified_registry_archive_with_source_async(request, &LiveNnsSource).await
}

/// Force-bootstrap and atomically publish one complete archive from an explicit async source.
///
/// Every returned report is reauthenticated locally against the built-in mainnet root key before
/// publication. A custom source therefore cannot acquire archive authority by merely satisfying
/// the structural report contract.
pub async fn bootstrap_nns_certified_registry_archive_with_source_async(
    request: &NnsCertifiedRegistryArchiveBootstrapRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveBootstrapError> {
    bootstrap_archive_with_authenticator_async(request, source, &BuiltInArchiveAuthenticator).await
}

pub(in crate::nns::registry::replay) async fn bootstrap_archive_with_authenticator_async(
    request: &NnsCertifiedRegistryArchiveBootstrapRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
    authenticator: &dyn ArchiveBatchAuthenticator,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveBootstrapError> {
    super::enforce_archive_mainnet_network(&request.bootstrap.network)?;
    let manifest_path = nns_certified_registry_archive_manifest_path(&request.archive_root);
    let lock_path = nns_certified_registry_archive_refresh_lock_path(&request.archive_root);
    create_managed_parent_directory(&request.cache_root, &manifest_path)
        .map_err(archive_filesystem_error)?;

    with_refresh_lock_async(
        RefreshLockRequest {
            cache_root: &request.cache_root,
            lock_path: &lock_path,
            target_path: &manifest_path,
            network: &request.bootstrap.network,
            now_unix_secs: request.bootstrap.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        archive_filesystem_error,
        || collect_and_publish(request, source, authenticator),
    )
    .await
}

async fn collect_and_publish(
    request: &NnsCertifiedRegistryArchiveBootstrapRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
    authenticator: &dyn ArchiveBatchAuthenticator,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveBootstrapError> {
    let (maximum_batch_query_calls, maximum_batch_response_bytes) =
        super::super::bootstrap::batch_reservation()?;
    let mut publisher = NnsCertifiedRegistryArchivePublisher::new(
        &request.cache_root,
        &request.archive_root,
        request.bootstrap.limits,
        request.storage_limits,
    );
    loop {
        publisher.ensure_next_batch_slot()?;
        let replay = publisher.replay_session();
        replay.ensure_next_source_call_capacity(
            maximum_batch_query_calls,
            maximum_batch_response_bytes,
        )?;
        let batch_request = NnsCertifiedRegistryDeltaBatchRequest::new(
            &request.bootstrap.network,
            &request.bootstrap.source_endpoint,
            replay.state().through_version(),
            request.bootstrap.now_unix_secs,
        );
        let report =
            fetch_nns_certified_registry_delta_batch_with_source_async(&batch_request, source)
                .await
                .map_err(NnsRegistryReplayError::from)?;
        let authenticated = authenticator
            .authenticate(&batch_request, &report)
            .map_err(
                |source| NnsCertifiedRegistryArchiveBootstrapError::BatchAuthentication {
                    requested_version: batch_request.requested_version,
                    source,
                },
            )?;
        publisher.apply_batch(&authenticated)?;
        if publisher.replay_session().is_complete() {
            return publisher.finish().map_err(Into::into);
        }
    }
}

const fn archive_filesystem_error(
    source: CacheFileError,
) -> NnsCertifiedRegistryArchiveBootstrapError {
    NnsCertifiedRegistryArchiveBootstrapError::Storage(
        NnsCertifiedRegistryArchiveStorageError::FileOperation { source },
    )
}
