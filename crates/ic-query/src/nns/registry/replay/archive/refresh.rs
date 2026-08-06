//! Module: nns::registry::replay::archive::refresh
//!
//! Responsibility: extend one existing certified Registry archive under its refresh lock.
//! Does not own: force bootstrap, read-through policy, default paths, cleanup, or CLI.
//! Boundary: resume, source collection, authentication, and manifest publication share one lock.

use super::{
    NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveStorageError,
    NnsCertifiedRegistryArchiveStorageLimits, nns_certified_registry_archive_manifest_path,
    nns_certified_registry_archive_refresh_lock_path,
    storage::{ArchiveBatchAuthenticator, BuiltInArchiveAuthenticator},
};
use crate::{
    cache_file::{
        CacheFileError, RefreshLockRequest, managed_file_exists, with_refresh_lock_async,
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
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// NnsCertifiedRegistryArchiveRefreshRequest
///
/// Explicit live collection, existing archive, locking, and cumulative storage policy.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryArchiveRefreshRequest {
    /// Mainnet source, observation time, and cumulative replay ceilings.
    pub collection: NnsCertifiedRegistryBootstrapRequest,
    /// Capability root that confines the archive and its refresh lock.
    pub cache_root: PathBuf,
    /// Caller-selected existing archive directory beneath `cache_root`.
    pub archive_root: PathBuf,
    /// Explicit cumulative manifest and retained-report storage ceilings.
    pub storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
    /// Age after which an abandoned archive refresh lock is reported as stale.
    pub lock_stale_after_seconds: u64,
}

impl NnsCertifiedRegistryArchiveRefreshRequest {
    /// Create an explicit incremental-refresh request without selecting paths or limits by default.
    #[must_use]
    pub fn new(
        collection: NnsCertifiedRegistryBootstrapRequest,
        cache_root: impl Into<PathBuf>,
        archive_root: impl Into<PathBuf>,
        storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            collection,
            cache_root: cache_root.into(),
            archive_root: archive_root.into(),
            storage_limits,
            lock_stale_after_seconds,
        }
    }
}

///
/// NnsCertifiedRegistryArchiveRefreshError
///
/// Typed replay, authentication, archive, and managed-filesystem failures from refresh.
///

#[derive(Debug, ThisError)]
pub enum NnsCertifiedRegistryArchiveRefreshError {
    /// Source evidence or bounded exact-target replay failed.
    #[error(transparent)]
    Replay(#[from] NnsRegistryReplayError),

    /// A structurally valid source report failed local mainnet evidence authentication.
    #[error(
        "certified Registry archive refresh batch after version {requested_version} failed local authentication: {source}"
    )]
    BatchAuthentication {
        /// Registry version after which the rejected report requested changes.
        requested_version: u64,
        /// Typed certificate, witness, chunk, or committed-content failure.
        #[source]
        source: NnsRegistryHostError,
    },

    /// Confined archive loading, publication, or refresh-lock management failed.
    #[error(transparent)]
    Storage(#[from] NnsCertifiedRegistryArchiveStorageError),
}

/// Refresh an existing complete archive from the live mainnet source.
///
/// This explicit operation requires an existing manifest, holds the archive's dedicated lock
/// across local reauthentication and every bounded source call, and publishes one complete
/// successor segment. It is never invoked by an archive load or ordinary catalog read-through.
pub async fn refresh_nns_certified_registry_archive_async(
    request: &NnsCertifiedRegistryArchiveRefreshRequest,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveRefreshError> {
    refresh_nns_certified_registry_archive_with_source_async(request, &LiveNnsSource).await
}

/// Refresh an existing complete archive from an explicit async source.
///
/// Every returned report is reauthenticated locally against the built-in mainnet root key before
/// publication. A custom source therefore cannot acquire archive authority by merely satisfying
/// the structural report contract.
pub async fn refresh_nns_certified_registry_archive_with_source_async(
    request: &NnsCertifiedRegistryArchiveRefreshRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveRefreshError> {
    refresh_archive_with_authenticator_async(request, source, &BuiltInArchiveAuthenticator).await
}

pub(in crate::nns::registry::replay) async fn refresh_archive_with_authenticator_async(
    request: &NnsCertifiedRegistryArchiveRefreshRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
    authenticator: &dyn ArchiveBatchAuthenticator,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveRefreshError> {
    super::enforce_archive_mainnet_network(&request.collection.network)?;
    let manifest_path = nns_certified_registry_archive_manifest_path(&request.archive_root);
    if !managed_file_exists(&request.cache_root, &manifest_path)
        .map_err(archive_filesystem_error)?
    {
        return Err(NnsCertifiedRegistryArchiveStorageError::MissingManifest {
            path: manifest_path,
        }
        .into());
    }
    let lock_path = nns_certified_registry_archive_refresh_lock_path(&request.archive_root);

    with_refresh_lock_async(
        RefreshLockRequest {
            cache_root: &request.cache_root,
            lock_path: &lock_path,
            target_path: &manifest_path,
            network: &request.collection.network,
            now_unix_secs: request.collection.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        archive_filesystem_error,
        || collect_and_publish_extension(request, source, authenticator),
    )
    .await
}

async fn collect_and_publish_extension(
    request: &NnsCertifiedRegistryArchiveRefreshRequest,
    source: &dyn NnsCertifiedRegistryDeltaSource,
    authenticator: &dyn ArchiveBatchAuthenticator,
) -> Result<NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveRefreshError> {
    let (maximum_batch_query_calls, maximum_batch_response_bytes) =
        super::super::bootstrap::batch_reservation()?;
    let mut publisher = super::storage::resume_archive_publisher_with_authenticator(
        &request.cache_root,
        &request.archive_root,
        request.collection.limits,
        request.storage_limits,
        authenticator,
    )?;
    loop {
        publisher.ensure_next_batch_slot()?;
        let replay = publisher.replay_session();
        if replay.is_complete() {
            replay.ensure_next_extension_source_call_capacity(
                maximum_batch_query_calls,
                maximum_batch_response_bytes,
            )?;
        } else {
            replay.ensure_next_source_call_capacity(
                maximum_batch_query_calls,
                maximum_batch_response_bytes,
            )?;
        }
        let batch_request = NnsCertifiedRegistryDeltaBatchRequest::new(
            &request.collection.network,
            &request.collection.source_endpoint,
            replay.state().through_version(),
            request.collection.now_unix_secs,
        );
        let report =
            fetch_nns_certified_registry_delta_batch_with_source_async(&batch_request, source)
                .await
                .map_err(NnsRegistryReplayError::from)?;
        let authenticated = authenticator
            .authenticate(&batch_request, &report)
            .map_err(
                |source| NnsCertifiedRegistryArchiveRefreshError::BatchAuthentication {
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
) -> NnsCertifiedRegistryArchiveRefreshError {
    NnsCertifiedRegistryArchiveRefreshError::Storage(
        NnsCertifiedRegistryArchiveStorageError::FileOperation { source },
    )
}
