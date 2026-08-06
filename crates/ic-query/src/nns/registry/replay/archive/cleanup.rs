//! Module: nns::registry::replay::archive::cleanup
//!
//! Responsibility: remove bounded unreferenced objects from one authenticated archive.
//! Does not own: refresh, force bootstrap, default paths, automatic policy, or CLI.
//! Boundary: authenticate first, classify exactly one object directory, then delete under lock.

use super::{
    NnsAuthenticatedRegistryArchive, NnsCertifiedRegistryArchiveStorageError,
    NnsCertifiedRegistryArchiveStorageLimits, nns_certified_registry_archive_manifest_path,
    nns_certified_registry_archive_refresh_lock_path,
    storage::{
        ArchiveBatchAuthenticator, BuiltInArchiveAuthenticator,
        load_nns_certified_registry_archive_with_authenticator,
        nns_certified_registry_archive_objects_path,
    },
};
use crate::{
    cache_file::{
        CacheFileError, ManagedDirectoryFile, RefreshLockRequest, managed_file_exists,
        remove_managed_regular_file, scan_managed_directory_files, with_refresh_lock,
    },
    nns::registry::NnsRegistryReplaySessionLimits,
    subnet_catalog::MAINNET_NETWORK,
};
use std::{cell::Cell, collections::BTreeSet, path::PathBuf};
use thiserror::Error as ThisError;

///
/// NnsCertifiedRegistryArchiveCleanupLimits
///
/// Explicit discovery and deletion ceilings for one orphan cleanup operation.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryArchiveCleanupLimits {
    /// Maximum regular files accepted in the exact archive object directory.
    pub max_scanned_objects: u64,
    /// Maximum unreferenced objects removed by one operation.
    pub max_removed_objects: u64,
    /// Maximum unreferenced file bytes removed by one operation.
    pub max_removed_bytes: u64,
}

impl NnsCertifiedRegistryArchiveCleanupLimits {
    /// Create explicit cleanup ceilings without selecting archive or filesystem defaults.
    #[must_use]
    pub const fn new(
        max_scanned_objects: u64,
        max_removed_objects: u64,
        max_removed_bytes: u64,
    ) -> Self {
        Self {
            max_scanned_objects,
            max_removed_objects,
            max_removed_bytes,
        }
    }
}

///
/// NnsCertifiedRegistryArchiveCleanupRequest
///
/// Existing archive, cumulative authentication limits, lock policy, and cleanup ceilings.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryArchiveCleanupRequest {
    /// Caller observation time recorded by the archive maintenance lock.
    pub now_unix_secs: u64,
    /// Capability root that confines the archive and its refresh lock.
    pub cache_root: PathBuf,
    /// Caller-selected existing archive directory beneath `cache_root`.
    pub archive_root: PathBuf,
    /// Cumulative replay ceilings used while reauthenticating the retained archive.
    pub replay_limits: NnsRegistryReplaySessionLimits,
    /// Cumulative manifest and retained-report storage ceilings used during authentication.
    pub storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
    /// Explicit exact-directory discovery and orphan-deletion ceilings.
    pub cleanup_limits: NnsCertifiedRegistryArchiveCleanupLimits,
    /// Age after which an abandoned archive maintenance lock is reported as stale.
    pub lock_stale_after_seconds: u64,
}

impl NnsCertifiedRegistryArchiveCleanupRequest {
    /// Create an explicit local cleanup request without selecting paths or limits by default.
    #[must_use]
    pub fn new(
        now_unix_secs: u64,
        cache_root: impl Into<PathBuf>,
        archive_root: impl Into<PathBuf>,
        replay_limits: NnsRegistryReplaySessionLimits,
        storage_limits: NnsCertifiedRegistryArchiveStorageLimits,
        cleanup_limits: NnsCertifiedRegistryArchiveCleanupLimits,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            now_unix_secs,
            cache_root: cache_root.into(),
            archive_root: archive_root.into(),
            replay_limits,
            storage_limits,
            cleanup_limits,
            lock_stale_after_seconds,
        }
    }
}

///
/// NnsCertifiedRegistryArchiveCleanupReport
///
/// Authenticated retained archive and exact results from one completed bounded cleanup.
///

#[derive(Debug, Eq, PartialEq)]
pub struct NnsCertifiedRegistryArchiveCleanupReport {
    /// Reauthenticated archive whose manifest defined the retained object set.
    pub archive: NnsAuthenticatedRegistryArchive,
    /// Regular files discovered in the exact object directory before deletion.
    pub scanned_object_count: u64,
    /// Unique content-addressed objects referenced by the authenticated manifest.
    pub referenced_object_count: u64,
    /// Unreferenced regular files removed by this operation.
    pub removed_object_count: u64,
    /// Total bytes reported for the removed unreferenced files.
    pub removed_bytes: u64,
}

///
/// NnsCertifiedRegistryArchiveCleanupError
///
/// Typed authentication, limit, confinement, lock, and partial-cleanup failures.
///

#[derive(Debug, ThisError)]
pub enum NnsCertifiedRegistryArchiveCleanupError {
    /// Existing archive loading or authentication failed before object classification.
    #[error(transparent)]
    Storage(#[from] NnsCertifiedRegistryArchiveStorageError),

    /// Discovery or deletion crossed a caller-selected cleanup ceiling before deletion began.
    #[error("certified Registry archive cleanup {field} is {actual}; caller maximum is {maximum}")]
    LimitExceeded {
        /// Bounded cleanup resource.
        field: &'static str,
        /// Caller-selected ceiling.
        maximum: u64,
        /// Observed or minimally required amount.
        actual: u64,
    },

    /// A confined filesystem or refresh-lock operation failed.
    #[error(
        "certified Registry archive cleanup failed after removing {removed_object_count} objects and {removed_bytes} bytes: {source}"
    )]
    FileOperation {
        /// Objects successfully removed before the failure.
        removed_object_count: u64,
        /// Bytes attributed to successfully removed objects before the failure.
        removed_bytes: u64,
        /// Underlying confined filesystem or lock failure.
        #[source]
        source: CacheFileError,
    },

    /// A classified orphan disappeared before its locked deletion.
    #[error(
        "certified Registry archive orphan {} disappeared after removing {removed_object_count} objects and {removed_bytes} bytes",
        path.display()
    )]
    OrphanDisappeared {
        /// Orphan path that was no longer present.
        path: PathBuf,
        /// Objects successfully removed before the disappearance.
        removed_object_count: u64,
        /// Bytes attributed to successfully removed objects before the disappearance.
        removed_bytes: u64,
    },

    /// Integer accounting could not be represented safely.
    #[error("certified Registry archive cleanup accounting overflow")]
    Accounting,
}

/// Authenticate an existing archive and remove its bounded unreferenced objects under lock.
///
/// This operation makes no network call. It scans only the archive's exact `objects` directory,
/// validates every entry as a confined regular file, applies all ceilings before the first
/// deletion, and returns the authenticated retained archive with exact deletion accounting.
pub fn cleanup_nns_certified_registry_archive(
    request: &NnsCertifiedRegistryArchiveCleanupRequest,
) -> Result<NnsCertifiedRegistryArchiveCleanupReport, NnsCertifiedRegistryArchiveCleanupError> {
    cleanup_archive_with_authenticator(request, &BuiltInArchiveAuthenticator)
}

pub(in crate::nns::registry::replay) fn cleanup_archive_with_authenticator(
    request: &NnsCertifiedRegistryArchiveCleanupRequest,
    authenticator: &dyn ArchiveBatchAuthenticator,
) -> Result<NnsCertifiedRegistryArchiveCleanupReport, NnsCertifiedRegistryArchiveCleanupError> {
    let manifest_path = nns_certified_registry_archive_manifest_path(&request.archive_root);
    if !managed_file_exists(&request.cache_root, &manifest_path).map_err(file_operation)? {
        return Err(NnsCertifiedRegistryArchiveStorageError::MissingManifest {
            path: manifest_path,
        }
        .into());
    }
    let lock_path = nns_certified_registry_archive_refresh_lock_path(&request.archive_root);
    let completed_removal = Cell::new((0_u64, 0_u64));
    with_refresh_lock(
        RefreshLockRequest {
            cache_root: &request.cache_root,
            lock_path: &lock_path,
            target_path: &manifest_path,
            network: MAINNET_NETWORK,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        |source| {
            let (removed_object_count, removed_bytes) = completed_removal.get();
            NnsCertifiedRegistryArchiveCleanupError::FileOperation {
                removed_object_count,
                removed_bytes,
                source,
            }
        },
        || {
            let report = cleanup_locked(request, authenticator)?;
            completed_removal.set((report.removed_object_count, report.removed_bytes));
            Ok(report)
        },
    )
}

fn cleanup_locked(
    request: &NnsCertifiedRegistryArchiveCleanupRequest,
    authenticator: &dyn ArchiveBatchAuthenticator,
) -> Result<NnsCertifiedRegistryArchiveCleanupReport, NnsCertifiedRegistryArchiveCleanupError> {
    let archive = load_nns_certified_registry_archive_with_authenticator(
        &request.cache_root,
        &request.archive_root,
        request.replay_limits,
        request.storage_limits,
        authenticator,
    )?;
    let objects_path = nns_certified_registry_archive_objects_path(&request.archive_root);
    let scan = scan_managed_directory_files(
        &request.cache_root,
        &objects_path,
        request.cleanup_limits.max_scanned_objects,
    )
    .map_err(file_operation)?;
    if scan.truncated {
        return Err(NnsCertifiedRegistryArchiveCleanupError::LimitExceeded {
            field: "scanned object count",
            maximum: request.cleanup_limits.max_scanned_objects,
            actual: checked_add(request.cleanup_limits.max_scanned_objects, 1)?,
        });
    }

    let referenced = archive
        .manifest()
        .batches
        .iter()
        .map(|batch| objects_path.join(format!("{}.json", batch.report_sha256)))
        .collect::<BTreeSet<_>>();
    let orphans = scan
        .files
        .iter()
        .filter(|object| !referenced.contains(&object.path))
        .collect::<Vec<_>>();
    let scanned_object_count = count(scan.files.len())?;
    let referenced_object_count = count(referenced.len())?;
    let removed_object_count = count(orphans.len())?;
    enforce_limit(
        "removed object count",
        removed_object_count,
        request.cleanup_limits.max_removed_objects,
    )?;
    let removed_bytes = orphans
        .iter()
        .try_fold(0_u64, |total, object| checked_add(total, object.bytes))?;
    enforce_limit(
        "removed object bytes",
        removed_bytes,
        request.cleanup_limits.max_removed_bytes,
    )?;

    remove_orphans(request, &orphans)?;
    Ok(NnsCertifiedRegistryArchiveCleanupReport {
        archive,
        scanned_object_count,
        referenced_object_count,
        removed_object_count,
        removed_bytes,
    })
}

fn remove_orphans(
    request: &NnsCertifiedRegistryArchiveCleanupRequest,
    orphans: &[&ManagedDirectoryFile],
) -> Result<(), NnsCertifiedRegistryArchiveCleanupError> {
    let mut removed_object_count = 0_u64;
    let mut removed_bytes = 0_u64;
    for orphan in orphans {
        let removed = match remove_managed_regular_file(&request.cache_root, &orphan.path) {
            Ok(removed) => removed,
            Err(failure) => {
                if failure.removed {
                    removed_object_count = checked_add(removed_object_count, 1)?;
                    removed_bytes = checked_add(removed_bytes, orphan.bytes)?;
                }
                return Err(NnsCertifiedRegistryArchiveCleanupError::FileOperation {
                    removed_object_count,
                    removed_bytes,
                    source: failure.source,
                });
            }
        };
        if !removed {
            return Err(NnsCertifiedRegistryArchiveCleanupError::OrphanDisappeared {
                path: orphan.path.clone(),
                removed_object_count,
                removed_bytes,
            });
        }
        removed_object_count = checked_add(removed_object_count, 1)?;
        removed_bytes = checked_add(removed_bytes, orphan.bytes)?;
    }
    Ok(())
}

const fn enforce_limit(
    field: &'static str,
    actual: u64,
    maximum: u64,
) -> Result<(), NnsCertifiedRegistryArchiveCleanupError> {
    if actual > maximum {
        return Err(NnsCertifiedRegistryArchiveCleanupError::LimitExceeded {
            field,
            maximum,
            actual,
        });
    }
    Ok(())
}

fn count(value: usize) -> Result<u64, NnsCertifiedRegistryArchiveCleanupError> {
    u64::try_from(value).map_err(|_| NnsCertifiedRegistryArchiveCleanupError::Accounting)
}

fn checked_add(left: u64, right: u64) -> Result<u64, NnsCertifiedRegistryArchiveCleanupError> {
    left.checked_add(right)
        .ok_or(NnsCertifiedRegistryArchiveCleanupError::Accounting)
}

const fn file_operation(source: CacheFileError) -> NnsCertifiedRegistryArchiveCleanupError {
    NnsCertifiedRegistryArchiveCleanupError::FileOperation {
        removed_object_count: 0,
        removed_bytes: 0,
        source,
    }
}
