use crate::cache_file::{
    CacheFileError, RefreshLockRequest, create_parent_directory, with_refresh_lock,
};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockedSnapshotRefreshRequest<'a> {
    pub snapshot_path: &'a Path,
    pub refresh_lock_path: &'a Path,
    pub network: &'a str,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockedSnapshotRefreshState {
    pub replaced_existing_snapshot: bool,
}

pub fn with_locked_snapshot_refresh<T, Error>(
    request: LockedSnapshotRefreshRequest<'_>,
    cache_error: impl Fn(CacheFileError) -> Error,
    action: impl FnOnce(LockedSnapshotRefreshState) -> Result<T, Error>,
) -> Result<T, Error> {
    create_parent_directory(request.snapshot_path).map_err(&cache_error)?;
    let state = LockedSnapshotRefreshState {
        replaced_existing_snapshot: request.snapshot_path.is_file(),
    };
    with_refresh_lock(
        RefreshLockRequest {
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
