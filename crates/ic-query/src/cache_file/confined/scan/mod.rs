//! Module: cache_file::confined::scan
//!
//! Responsibility: bounded confined-cache discovery.
//! Does not own: cache schemas, path confinement, or atomic publication.
//! Boundary: traverses capability-rooted directories without following symbolic links.

use super::CacheFileError;
#[cfg(unix)]
use super::capability_error;
#[cfg(all(unix, feature = "host"))]
use super::directory_options;
#[cfg(all(unix, feature = "host"))]
use super::open_managed_path_error;
#[cfg(all(unix, feature = "sns-host"))]
use super::validate_managed_file_mode;
#[cfg(unix)]
use super::{ConfinedCacheRoot, confinement_error};
#[cfg(all(unix, feature = "host"))]
use ic_host_fs::capability::CapabilityError;
#[cfg(unix)]
use ic_host_fs::capability::EntryKind;
#[cfg(all(unix, feature = "host"))]
use std::io;
use std::path::{Path, PathBuf};

///
/// ManagedFileScan
///
/// Bounded capability-rooted discovery result for selected managed files.
///

#[cfg(feature = "host")]
#[derive(Debug, Default)]
pub struct ManagedFileScan {
    /// Whether the selected cache root existed.
    pub root_found: bool,
    /// Canonically ordered selected regular file paths.
    pub paths: Vec<PathBuf>,
    /// Whether discovery reached its selected-file, visited-entry, or depth bound.
    pub truncated: bool,
}

/// Traverse a cache root without following links and retain selected regular files.
#[cfg(feature = "host")]
pub fn collect_managed_files(
    cache_root: &Path,
    limit: usize,
    include: impl Fn(&Path) -> bool,
) -> Result<ManagedFileScan, CacheFileError> {
    #[cfg(not(unix))]
    {
        let _ = (cache_root, limit, include);
        Err(super::unsupported_platform())
    }
    #[cfg(unix)]
    {
        let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
            return Ok(ManagedFileScan::default());
        };
        // Queue paths, not open capabilities: sibling count cannot exhaust descriptors.
        let mut directories = vec![(root.display_root.clone(), 0usize)];
        let entry_limit = limit.saturating_mul(16);
        let mut visited = 0usize;
        let mut scan = ManagedFileScan {
            root_found: true,
            ..ManagedFileScan::default()
        };
        while let Some((display_directory, depth)) = directories.pop() {
            let Some(managed) =
                root.resolve_parent(&display_directory.join(".icq-directory-probe"), false)?
            else {
                continue;
            };
            let directory = managed.entry.parent();
            let entries = directory
                .entries(entry_limit.saturating_sub(visited))
                .map_err(|error| capability_error(cache_root, error))?;
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(CapabilityError::EntryLimit { .. }) => {
                        scan.truncated = true;
                        scan.paths.sort();
                        return Ok(scan);
                    }
                    Err(error) => return Err(capability_error(cache_root, error)),
                };
                visited += 1;
                let name = entry.name;
                let path = display_directory.join(&name);
                if entry.kind == EntryKind::SymbolicLink {
                    return Err(confinement_error(
                        cache_root,
                        &path,
                        "managed cache entry is a symbolic link",
                    ));
                }
                if entry.kind == EntryKind::Directory {
                    let child = directory
                        .open_directory(&name, directory_options(false))
                        .map_err(|error| capability_error(cache_root, error))?
                        .ok_or_else(|| {
                            open_managed_path_error(
                                cache_root,
                                &path,
                                io::Error::new(
                                    io::ErrorKind::NotFound,
                                    "managed directory disappeared during discovery",
                                ),
                            )
                        })?;
                    drop(child);
                    if depth == 32 {
                        scan.truncated = true;
                    } else {
                        directories.push((path, depth + 1));
                    }
                    continue;
                }
                if entry.kind != EntryKind::File {
                    return Err(confinement_error(
                        cache_root,
                        &path,
                        "managed cache entry is not a regular file or directory",
                    ));
                }
                let Some(managed_path) = root.resolve_parent(&path, false)? else {
                    continue;
                };
                let Some(file) = managed_path.open_regular_file()? else {
                    continue;
                };
                drop(file);
                if !include(&path) {
                    continue;
                }
                if scan.paths.len() == limit {
                    scan.truncated = true;
                    scan.paths.sort();
                    return Ok(scan);
                }
                scan.paths.push(path);
            }
        }
        scan.paths.sort();
        Ok(scan)
    }
}

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

#[cfg(all(test, feature = "host"))]
mod tests;
