//! Module: cache::status::locks
//!
//! Responsibility: inspect refresh locks and project their generic status rows.
//! Does not own: lock acquisition, directory traversal, or operator cleanup.
//! Boundary: evaluates only serialized lock evidence and caller-supplied time.

use super::{
    super::{CacheRefreshLockStatus, CacheRefreshLockStatusRow},
    header::component_from_path,
};
use crate::{
    cache_file::{RefreshLockEvidence, inspect_refresh_lock},
    subnet_catalog::format_utc_timestamp_secs,
};
use std::path::Path;

struct RefreshLockRowContext<'path> {
    root: &'path Path,
    path: &'path Path,
    relative: &'path Path,
    relative_path: String,
    size_bytes: u64,
}

impl<'path> RefreshLockRowContext<'path> {
    fn new(root: &'path Path, path: &'path Path) -> Self {
        let relative = path.strip_prefix(root).unwrap_or(path);
        Self {
            root,
            path,
            relative,
            relative_path: relative.display().to_string(),
            size_bytes: path.metadata().map_or(0, |metadata| metadata.len()),
        }
    }
}

pub(super) fn refresh_lock_status_row(
    root: &Path,
    path: &Path,
    now_unix_secs: u64,
) -> CacheRefreshLockStatusRow {
    let context = RefreshLockRowContext::new(root, path);
    let evidence = match inspect_refresh_lock(path) {
        Ok(evidence) => evidence,
        Err(error) => {
            return invalid_refresh_lock_row(context, error.to_string());
        }
    };
    let now_unix_ms = now_unix_secs.saturating_mul(1_000);
    let Some(age_unix_ms) = now_unix_ms.checked_sub(evidence.started_at_unix_ms) else {
        return invalid_refresh_lock_evidence_row(
            context,
            evidence,
            "refresh lock acquisition timestamp is in the future".to_string(),
        );
    };
    let status = if age_unix_ms > evidence.stale_after_seconds.saturating_mul(1_000) {
        CacheRefreshLockStatus::Stale
    } else {
        CacheRefreshLockStatus::Active
    };
    refresh_lock_evidence_row(context, evidence, status, Some(age_unix_ms / 1_000), None)
}

fn invalid_refresh_lock_row(
    context: RefreshLockRowContext<'_>,
    error: String,
) -> CacheRefreshLockStatusRow {
    CacheRefreshLockStatusRow {
        component: component_from_path(context.relative),
        refresh_lock_path: context.path.display().to_string(),
        relative_path: context.relative_path,
        status: CacheRefreshLockStatus::Invalid,
        schema_version: None,
        network: None,
        pid: None,
        started_at_unix_ms: None,
        started_at: None,
        age_seconds: None,
        stale_after_seconds: None,
        target_path: None,
        size_bytes: context.size_bytes,
        error: Some(error),
    }
}

fn invalid_refresh_lock_evidence_row(
    context: RefreshLockRowContext<'_>,
    evidence: RefreshLockEvidence,
    error: String,
) -> CacheRefreshLockStatusRow {
    refresh_lock_evidence_row(
        context,
        evidence,
        CacheRefreshLockStatus::Invalid,
        None,
        Some(error),
    )
}

fn refresh_lock_evidence_row(
    context: RefreshLockRowContext<'_>,
    evidence: RefreshLockEvidence,
    status: CacheRefreshLockStatus,
    age_seconds: Option<u64>,
    error: Option<String>,
) -> CacheRefreshLockStatusRow {
    let target = Path::new(&evidence.target_path);
    let component_path = target
        .strip_prefix(context.root)
        .unwrap_or(context.relative);
    CacheRefreshLockStatusRow {
        component: component_from_path(component_path),
        refresh_lock_path: context.path.display().to_string(),
        relative_path: context.relative_path,
        status,
        schema_version: Some(evidence.schema_version),
        network: Some(evidence.network),
        pid: Some(evidence.pid),
        started_at_unix_ms: Some(evidence.started_at_unix_ms),
        started_at: Some(format_utc_timestamp_secs(
            evidence.started_at_unix_ms / 1_000,
        )),
        age_seconds,
        stale_after_seconds: Some(evidence.stale_after_seconds),
        target_path: Some(evidence.target_path),
        size_bytes: context.size_bytes,
        error,
    }
}
