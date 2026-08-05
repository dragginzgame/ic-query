//! Module: cache::status::discovery
//!
//! Responsibility: discover bounded cache and refresh-lock candidate paths.
//! Does not own: file parsing, row projection, refresh, or deletion.
//! Boundary: traverses the selected cache root without following symlinks.

use super::CacheStatusError;
use crate::cache_file::collect_managed_files;
use std::path::{Path, PathBuf};

pub(super) const CACHE_STATUS_SCAN_LIMIT: usize = 10_000;

///
/// CacheInventoryPaths
///
/// Canonically ordered cache and refresh-lock candidates from one bounded scan.
///

#[derive(Default)]
pub(super) struct CacheInventoryPaths {
    pub(super) caches: Vec<PathBuf>,
    pub(super) refresh_locks: Vec<PathBuf>,
    pub(super) root_found: bool,
    pub(super) truncated: bool,
}

enum CandidateKind {
    Cache,
    RefreshLock,
}

pub(super) fn collect_inventory_paths(
    root: &Path,
) -> Result<CacheInventoryPaths, CacheStatusError> {
    let scan = collect_managed_files(root, CACHE_STATUS_SCAN_LIMIT, |path| {
        candidate_kind(path).is_some()
    })?;
    let mut inventory = CacheInventoryPaths {
        root_found: scan.root_found,
        truncated: scan.truncated,
        ..CacheInventoryPaths::default()
    };
    for path in scan.paths {
        match candidate_kind(&path).expect("selected inventory path has a candidate kind") {
            CandidateKind::Cache => inventory.caches.push(path),
            CandidateKind::RefreshLock => inventory.refresh_locks.push(path),
        }
    }
    sort_inventory(&mut inventory);
    Ok(inventory)
}

fn candidate_kind(path: &Path) -> Option<CandidateKind> {
    let name = path.file_name()?.to_str()?;
    if matches!(
        name,
        "catalog.json"
            | "nodes.json"
            | "providers.json"
            | "operators.json"
            | "data-centers.json"
            | "report.json"
            | "full.json"
    ) {
        Some(CandidateKind::Cache)
    } else if name == "refresh.lock" || name.ends_with(".refresh.lock") {
        Some(CandidateKind::RefreshLock)
    } else {
        None
    }
}

fn sort_inventory(inventory: &mut CacheInventoryPaths) {
    inventory.caches.sort();
    inventory.refresh_locks.sort();
}
