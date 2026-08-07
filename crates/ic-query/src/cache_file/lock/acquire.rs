//! Module: cache_file::lock::acquire
//!
//! Responsibility: acquire refresh locks and reject active locks.
//! Does not own: guarded refresh execution or lock cleanup policy.
//! Boundary: creates, reads, validates, and reports stale lock files.

#[cfg(any(feature = "host", test))]
use super::model::RefreshLockEvidence;
use super::{
    guard::RefreshLockGuard,
    model::{REFRESH_LOCK_SCHEMA_VERSION, RefreshLockFile, RefreshLockRequest},
};
use crate::cache_file::{
    BoundedManagedFileReadError, CacheFileError,
    confined::{ConfinedManagedPath, managed_path_for_create},
};
use std::{io, io::Write, path::Path};

pub(super) const MAX_REFRESH_LOCK_BYTES: u64 = 64 * 1024;

pub(super) fn acquire_refresh_lock(
    request: RefreshLockRequest<'_>,
) -> Result<RefreshLockGuard, CacheFileError> {
    let now_unix_ms = request.now_unix_secs.saturating_mul(1_000);
    let lock_path = managed_path_for_create(request.cache_root, request.lock_path)?;
    match lock_path.create_new_file() {
        Ok(file) => {
            if let Err(err) = write_refresh_lock_file(file, request, now_unix_ms) {
                let _ = lock_path.remove_file();
                return Err(err);
            }
            lock_path.sync_parent()?;
            Ok(RefreshLockGuard::new(lock_path))
        }
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            let existing = read_refresh_lock(&lock_path)?;
            validate_refresh_lock(request, &existing)?;
            validate_refresh_lock_time(request.lock_path, &existing, now_unix_ms)?;
            if lock_is_stale(
                existing.started_at_unix_ms,
                now_unix_ms,
                existing.stale_after_seconds,
            ) {
                return Err(CacheFileError::StaleRefreshLock {
                    path: request.lock_path.to_path_buf(),
                    started_at_unix_ms: existing.started_at_unix_ms,
                });
            }
            Err(CacheFileError::RefreshAlreadyInProgress {
                path: request.lock_path.to_path_buf(),
                started_at_unix_ms: existing.started_at_unix_ms,
            })
        }
        Err(source) => Err(CacheFileError::CreateRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        }),
    }
}

fn write_refresh_lock_file(
    mut file: cap_std::fs::File,
    request: RefreshLockRequest<'_>,
    now_unix_ms: u64,
) -> Result<(), CacheFileError> {
    let lock = RefreshLockFile::new(request, now_unix_ms);
    let data = serde_json::to_vec_pretty(&lock).map_err(|source| {
        CacheFileError::SerializeRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        }
    })?;
    file.write_all(&data)
        .map_err(|source| CacheFileError::WriteRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        })?;
    file.sync_all()
        .map_err(|source| CacheFileError::WriteRefreshLock {
            path: request.lock_path.to_path_buf(),
            source,
        })
}

#[cfg(any(feature = "host", test))]
pub fn inspect_refresh_lock(
    cache_root: &Path,
    path: &Path,
) -> Result<RefreshLockEvidence, CacheFileError> {
    let managed = managed_path_for_create(cache_root, path)?;
    read_refresh_lock(&managed).map(Into::into)
}

fn read_refresh_lock(path: &ConfinedManagedPath) -> Result<RefreshLockFile, CacheFileError> {
    let display_path = path.display_path();
    let data = path
        .read_bounded(MAX_REFRESH_LOCK_BYTES)
        .map_err(refresh_lock_read_error)?
        .ok_or_else(|| CacheFileError::ReadRefreshLock {
            path: display_path.to_path_buf(),
            source: io::Error::new(io::ErrorKind::NotFound, "refresh lock disappeared"),
        })?;
    let lock: RefreshLockFile =
        serde_json::from_slice(&data).map_err(|source| CacheFileError::ParseRefreshLock {
            path: display_path.to_path_buf(),
            source,
        })?;
    validate_refresh_lock_shape(display_path, &lock)?;
    Ok(lock)
}

fn refresh_lock_read_error(error: BoundedManagedFileReadError) -> CacheFileError {
    match error {
        BoundedManagedFileReadError::Operation(source) => source,
        BoundedManagedFileReadError::Read { path, source } => {
            CacheFileError::ReadRefreshLock { path, source }
        }
        BoundedManagedFileReadError::LimitExceeded {
            path,
            actual,
            maximum,
        } => CacheFileError::InvalidRefreshLock {
            path,
            reason: format!("refresh lock is {actual} bytes, maximum is {maximum}"),
        },
        BoundedManagedFileReadError::Accounting { path } => CacheFileError::InvalidRefreshLock {
            path,
            reason: "refresh-lock byte length cannot be represented safely".to_string(),
        },
    }
}

fn validate_refresh_lock_shape(path: &Path, lock: &RefreshLockFile) -> Result<(), CacheFileError> {
    let reason = if lock.schema_version != REFRESH_LOCK_SCHEMA_VERSION {
        Some(format!(
            "schema_version is {}, expected {}",
            lock.schema_version, REFRESH_LOCK_SCHEMA_VERSION
        ))
    } else if lock.network.is_empty() {
        Some("network must not be empty".to_string())
    } else if lock.pid == 0 {
        Some("pid must be greater than zero".to_string())
    } else if lock.target_path.is_empty() {
        Some("target_path must not be empty".to_string())
    } else {
        None
    };
    reason.map_or(Ok(()), |reason| {
        Err(CacheFileError::InvalidRefreshLock {
            path: path.to_path_buf(),
            reason,
        })
    })
}

fn validate_refresh_lock(
    request: RefreshLockRequest<'_>,
    lock: &RefreshLockFile,
) -> Result<(), CacheFileError> {
    let reason = if lock.network != request.network {
        Some(format!(
            "network is {}, expected {}",
            lock.network, request.network
        ))
    } else if lock.target_path != request.target_path.display().to_string() {
        Some(format!(
            "target_path is {}, expected {}",
            lock.target_path,
            request.target_path.display()
        ))
    } else {
        None
    };
    reason.map_or(Ok(()), |reason| {
        Err(CacheFileError::InvalidRefreshLock {
            path: request.lock_path.to_path_buf(),
            reason,
        })
    })
}

fn validate_refresh_lock_time(
    path: &Path,
    lock: &RefreshLockFile,
    now_unix_ms: u64,
) -> Result<(), CacheFileError> {
    if lock.started_at_unix_ms <= now_unix_ms {
        return Ok(());
    }
    Err(CacheFileError::InvalidRefreshLock {
        path: path.to_path_buf(),
        reason: format!(
            "started_at_unix_ms is {}, later than current unix_ms={now_unix_ms}",
            lock.started_at_unix_ms
        ),
    })
}

fn lock_is_stale(started_at_unix_ms: u64, now_unix_ms: u64, stale_after_seconds: u64) -> bool {
    now_unix_ms
        .saturating_sub(started_at_unix_ms)
        .gt(&stale_after_seconds.saturating_mul(1_000))
}
