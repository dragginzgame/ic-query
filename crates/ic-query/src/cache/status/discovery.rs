//! Module: cache::status::discovery
//!
//! Responsibility: discover bounded cache and refresh-lock candidate paths.
//! Does not own: file parsing, row projection, refresh, or deletion.
//! Boundary: traverses the selected cache root without following symlinks.

use super::CacheStatusError;
use std::{fs, path::Path, path::PathBuf};

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
    pub(super) truncated: bool,
}

enum CandidateKind {
    Cache,
    RefreshLock,
}

pub(super) fn collect_inventory_paths(
    root: &Path,
) -> Result<CacheInventoryPaths, CacheStatusError> {
    let mut directories = vec![root.to_path_buf()];
    let mut inventory = CacheInventoryPaths::default();
    while let Some(directory) = directories.pop() {
        let entries =
            fs::read_dir(&directory).map_err(|source| CacheStatusError::ReadDirectory {
                path: directory.clone(),
                source,
            })?;
        for entry in entries {
            let entry = entry.map_err(|source| CacheStatusError::ReadDirectory {
                path: directory.clone(),
                source,
            })?;
            let file_type =
                entry
                    .file_type()
                    .map_err(|source| CacheStatusError::ReadDirectory {
                        path: directory.clone(),
                        source,
                    })?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_dir() {
                directories.push(path);
                continue;
            }
            let Some(kind) = file_type.is_file().then(|| candidate_kind(&path)).flatten() else {
                continue;
            };
            if inventory.caches.len() + inventory.refresh_locks.len() == CACHE_STATUS_SCAN_LIMIT {
                inventory.truncated = true;
                sort_inventory(&mut inventory);
                return Ok(inventory);
            }
            match kind {
                CandidateKind::Cache => inventory.caches.push(path),
                CandidateKind::RefreshLock => inventory.refresh_locks.push(path),
            }
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
