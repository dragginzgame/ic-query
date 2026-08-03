//! Module: cache::status
//!
//! Responsibility: assemble bounded local cache and refresh-lock inventory reports.
//! Does not own: cache refresh, family-specific schema validation, or deletion.
//! Boundary: coordinates local discovery and row projection without network access.

mod discovery;
mod header;
mod locks;
#[cfg(test)]
mod tests;

use super::{
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheFileStatus, CacheRefreshLockStatus,
    CacheRefreshLockStatusRow, CacheStatusReport, CacheStatusRequest, CacheStatusRow,
};
use crate::subnet_catalog::format_utc_timestamp_secs;
use discovery::{CACHE_STATUS_SCAN_LIMIT, CacheInventoryPaths, collect_inventory_paths};
use header::cache_status_row;
use locks::refresh_lock_status_row;
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// CacheStatusError
///
/// Filesystem failure encountered while inventorying the user-level cache root.
///

#[derive(Debug, ThisError)]
pub enum CacheStatusError {
    /// A cache directory could not be inspected.
    #[error("failed to inspect cache directory at {}: {source}", path.display())]
    ReadDirectory {
        /// Directory that could not be inspected.
        path: PathBuf,
        /// Underlying filesystem failure.
        source: std::io::Error,
    },
}

/// Build a bounded local-only inventory of every known cache and refresh lock.
pub fn build_cache_status_report(
    request: &CacheStatusRequest,
) -> Result<CacheStatusReport, CacheStatusError> {
    let cache_root_found = request.cache_root.is_dir();
    let inventory = if cache_root_found {
        collect_inventory_paths(&request.cache_root)?
    } else {
        CacheInventoryPaths::default()
    };
    let caches = inventory
        .caches
        .into_iter()
        .map(|path| cache_status_row(&request.cache_root, &path, request.now_unix_secs))
        .collect::<Vec<_>>();
    let refresh_locks = inventory
        .refresh_locks
        .into_iter()
        .map(|path| refresh_lock_status_row(&request.cache_root, &path, request.now_unix_secs))
        .collect::<Vec<_>>();
    Ok(CacheStatusReport {
        schema_version: CACHE_STATUS_REPORT_SCHEMA_VERSION,
        cache_root: request.cache_root.display().to_string(),
        inspected_at: format_utc_timestamp_secs(request.now_unix_secs),
        cache_root_found,
        scan_limit: CACHE_STATUS_SCAN_LIMIT,
        truncated: inventory.truncated,
        cache_count: caches.len(),
        fresh_count: count_cache_status(&caches, CacheFileStatus::Fresh),
        stale_count: count_cache_status(&caches, CacheFileStatus::Stale),
        unmanaged_count: count_cache_status(&caches, CacheFileStatus::Unmanaged),
        invalid_count: count_cache_status(&caches, CacheFileStatus::Invalid),
        total_size_bytes: caches.iter().map(|row| row.size_bytes).sum(),
        caches,
        refresh_lock_count: refresh_locks.len(),
        active_refresh_lock_count: count_refresh_lock_status(
            &refresh_locks,
            CacheRefreshLockStatus::Active,
        ),
        stale_refresh_lock_count: count_refresh_lock_status(
            &refresh_locks,
            CacheRefreshLockStatus::Stale,
        ),
        invalid_refresh_lock_count: count_refresh_lock_status(
            &refresh_locks,
            CacheRefreshLockStatus::Invalid,
        ),
        refresh_lock_size_bytes: refresh_locks.iter().map(|row| row.size_bytes).sum(),
        refresh_locks,
    })
}

fn count_cache_status(rows: &[CacheStatusRow], status: CacheFileStatus) -> usize {
    rows.iter().filter(|row| row.status == status).count()
}

fn count_refresh_lock_status(
    rows: &[CacheRefreshLockStatusRow],
    status: CacheRefreshLockStatus,
) -> usize {
    rows.iter().filter(|row| row.status == status).count()
}
