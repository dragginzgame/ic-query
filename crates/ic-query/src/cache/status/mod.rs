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
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheAgeStatus, CacheHeaderStatus, CacheRefreshLockStatus,
    CacheRefreshLockStatusRow, CacheStatusReport, CacheStatusRequest, CacheStatusRow,
};
use crate::{CacheFileError, subnet_catalog::format_utc_timestamp_secs};
use discovery::{CACHE_STATUS_SCAN_LIMIT, collect_inventory_paths};
use header::cache_status_row;
use locks::refresh_lock_status_row;
use thiserror::Error as ThisError;

///
/// CacheStatusError
///
/// Filesystem failure encountered while inventorying the user-level cache root.
///

#[derive(Debug, ThisError)]
pub enum CacheStatusError {
    /// Capability-rooted managed cache inspection failed.
    #[error(transparent)]
    CacheOperation(#[from] CacheFileError),
}

/// Build a bounded local-only inventory of every known cache and refresh lock.
pub fn build_cache_status_report(
    request: &CacheStatusRequest,
) -> Result<CacheStatusReport, CacheStatusError> {
    let inventory = collect_inventory_paths(&request.cache_root)?;
    let cache_root_found = inventory.root_found;
    let caches = inventory
        .caches
        .into_iter()
        .map(|path| cache_status_row(&request.cache_root, &path, request.now_unix_secs))
        .collect::<Result<Vec<_>, _>>()?;
    let refresh_locks = inventory
        .refresh_locks
        .into_iter()
        .map(|path| refresh_lock_status_row(&request.cache_root, &path, request.now_unix_secs))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CacheStatusReport {
        schema_version: CACHE_STATUS_REPORT_SCHEMA_VERSION,
        cache_root: request.cache_root.display().to_string(),
        inspected_at: format_utc_timestamp_secs(request.now_unix_secs),
        cache_root_found,
        scan_limit: CACHE_STATUS_SCAN_LIMIT,
        truncated: inventory.truncated,
        family_validation_performed: false,
        cache_count: caches.len(),
        readable_header_count: count_cache_header_status(&caches, CacheHeaderStatus::Readable),
        invalid_header_count: count_cache_header_status(&caches, CacheHeaderStatus::Invalid),
        fresh_count: count_cache_age_status(&caches, CacheAgeStatus::Fresh),
        stale_count: count_cache_age_status(&caches, CacheAgeStatus::Stale),
        unmanaged_age_count: count_cache_age_status(&caches, CacheAgeStatus::Unmanaged),
        unknown_age_count: count_cache_age_status(&caches, CacheAgeStatus::Unknown),
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

fn count_cache_header_status(rows: &[CacheStatusRow], status: CacheHeaderStatus) -> usize {
    rows.iter()
        .filter(|row| row.header_status == status)
        .count()
}

fn count_cache_age_status(rows: &[CacheStatusRow], status: CacheAgeStatus) -> usize {
    rows.iter().filter(|row| row.age_status == status).count()
}

fn count_refresh_lock_status(
    rows: &[CacheRefreshLockStatusRow],
    status: CacheRefreshLockStatus,
) -> usize {
    rows.iter().filter(|row| row.status == status).count()
}
