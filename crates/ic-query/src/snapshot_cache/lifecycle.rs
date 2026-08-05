//! Module: snapshot_cache::lifecycle
//!
//! Responsibility: coordinate snapshot refresh locks and attempt lifecycle hooks.
//! Does not own: paged fetching, snapshot JSON publication, or cache schemas.
//! Boundary: wraps command-owned refresh actions in shared lock/attempt sequencing.

use crate::cache_file::{
    CacheFileError, RefreshLockRequest, create_managed_parent_directory, managed_file_exists,
    with_refresh_lock,
};
use std::{fmt::Display, path::Path};

///
/// LockedSnapshotRefreshRequest
///
/// Inputs for refreshing one complete snapshot under a shared lock.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockedSnapshotRefreshRequest<'a> {
    /// Capability root that confines the snapshot and refresh lock.
    pub cache_root: &'a Path,
    pub snapshot_path: &'a Path,
    pub refresh_lock_path: &'a Path,
    pub network: &'a str,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
}

///
/// LockedSnapshotRefreshState
///
/// State derived after acquiring the snapshot refresh lock.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockedSnapshotRefreshState {
    pub replaced_existing_snapshot: bool,
}

pub fn with_locked_snapshot_refresh<T, Error>(
    request: LockedSnapshotRefreshRequest<'_>,
    cache_error: impl Fn(CacheFileError) -> Error,
    action: impl FnOnce(LockedSnapshotRefreshState) -> Result<T, Error>,
) -> Result<T, Error> {
    create_managed_parent_directory(request.cache_root, request.snapshot_path)
        .map_err(&cache_error)?;
    let state = LockedSnapshotRefreshState {
        replaced_existing_snapshot: managed_file_exists(request.cache_root, request.snapshot_path)
            .map_err(&cache_error)?,
    };
    with_refresh_lock(
        RefreshLockRequest {
            cache_root: request.cache_root,
            lock_path: request.refresh_lock_path,
            target_path: request.snapshot_path,
            network: request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        cache_error,
        || action(state),
    )
}

pub fn run_snapshot_refresh_with_attempts<Output, Error>(
    write_running_attempt: impl FnOnce() -> Result<(), Error>,
    run_refresh: impl FnOnce() -> Result<Output, Error>,
    write_failed_attempt: impl FnOnce(&Error),
) -> Result<Output, Error> {
    write_running_attempt()?;
    match run_refresh() {
        Ok(output) => Ok(output),
        Err(err) => {
            write_failed_attempt(&err);
            Err(err)
        }
    }
}

pub fn publish_snapshot_with_attempt<Error>(
    publish_snapshot: impl FnOnce() -> Result<(), Error>,
    finalize_attempt: impl FnOnce() -> Result<(), Error>,
) -> Result<Option<String>, Error>
where
    Error: Display,
{
    publish_snapshot()?;
    Ok(finalize_attempt().err().map(|err| err.to_string()))
}
