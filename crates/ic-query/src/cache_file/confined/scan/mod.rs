//! Module: cache_file::confined::scan
//!
//! Responsibility: bounded confined-cache discovery.
//! Does not own: cache schemas, path confinement, or atomic publication.
//! Boundary: traverses capability-rooted directories without following symbolic links.

use super::CacheFileError;
#[cfg(unix)]
use super::capability_error;
#[cfg(all(unix, feature = "sns-host"))]
use super::validate_managed_file_mode;
#[cfg(unix)]
use super::{ConfinedCacheRoot, confinement_error};
#[cfg(unix)]
use ic_host_fs::capability::EntryKind;
use std::path::{Path, PathBuf};

/// Discover canonical collection files beneath one confined network directory.
#[cfg(feature = "sns-host")]
pub fn collect_managed_collection_files(
    cache_root: &Path,
    network_dir: &Path,
    collection: &str,
    file_name: &str,
    maximum_files: usize,
) -> Result<Vec<PathBuf>, CacheFileError> {
    #[cfg(not(unix))]
    {
        let _ = (
            cache_root,
            network_dir,
            collection,
            file_name,
            maximum_files,
        );
        Err(super::unsupported_platform())
    }
    #[cfg(unix)]
    {
        let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
            return Ok(Vec::new());
        };
        let probe_path = network_dir.join(".icq-directory-probe");
        let Some(network) = root.resolve_parent(&probe_path, false)? else {
            return Ok(Vec::new());
        };
        let entries = network
            .entry
            .parent()
            .entries(maximum_files.saturating_mul(16))
            .map_err(|error| capability_error(cache_root, error))?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| capability_error(cache_root, error))?;
            let entity_path = network_dir.join(&entry.name);
            if entry.kind == EntryKind::SymbolicLink {
                return Err(confinement_error(
                    cache_root,
                    &entity_path,
                    "managed collection entity is a symbolic link",
                ));
            }
            if entry.kind != EntryKind::Directory {
                continue;
            }
            let candidate = entity_path.join(collection).join(file_name);
            let Some(candidate_path) = root.resolve_parent(&candidate, false)? else {
                continue;
            };
            if let Some(file) = candidate_path.open_regular_file()? {
                validate_managed_file_mode(&candidate, &file)?;
                if paths.len() == maximum_files {
                    return Err(CacheFileError::ScanLimitExceeded {
                        path: network_dir.to_path_buf(),
                        resource: "collection candidates",
                        maximum: maximum_files as u64,
                    });
                }
                paths.push(candidate);
            }
        }
        paths.sort();
        Ok(paths)
    }
}

#[cfg(all(test, feature = "sns-host"))]
mod tests;
