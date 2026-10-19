//! Module: cache_file::confined::scan
//!
//! Responsibility: bounded confined-cache discovery and regular-file removal.
//! Does not own: cache schemas, path confinement, or atomic publication.
//! Boundary: traverses capability-rooted directories without following symbolic links.

use super::CacheFileError;
#[cfg(unix)]
use super::capability_error;
#[cfg(all(unix, feature = "host"))]
use super::directory_options;
#[cfg(all(unix, any(feature = "host", feature = "certified-subnet-catalog-host")))]
use super::open_managed_path_error;
#[cfg(all(unix, feature = "sns-host"))]
use super::validate_managed_file_mode;
#[cfg(unix)]
use super::{ConfinedCacheRoot, confinement_error};
#[cfg(all(unix, any(feature = "host", feature = "certified-subnet-catalog-host")))]
use ic_host_fs::capability::CapabilityError;
#[cfg(unix)]
use ic_host_fs::capability::EntryKind;
#[cfg(all(unix, any(feature = "certified-subnet-catalog-host", feature = "host")))]
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

///
/// ManagedDirectoryFile
///
/// Validated regular file discovered directly beneath one managed directory.
///

#[cfg(feature = "certified-subnet-catalog-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedDirectoryFile {
    /// Confined display path beneath the caller-selected cache root.
    pub path: PathBuf,
    /// File length reported after opening and validating the regular file.
    pub bytes: u64,
}

///
/// ManagedDirectoryScan
///
/// Bounded exact-directory scan that never traverses child directories.
///

#[cfg(feature = "certified-subnet-catalog-host")]
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ManagedDirectoryScan {
    /// Canonically ordered validated regular files.
    pub files: Vec<ManagedDirectoryFile>,
    /// Whether another directory entry existed beyond the caller's ceiling.
    pub truncated: bool,
}

///
/// ManagedFileRemovalError
///
/// Confined removal failure that records whether unlinking completed before a sync error.
///

#[cfg(feature = "certified-subnet-catalog-host")]
#[derive(Debug)]
pub struct ManagedFileRemovalError {
    /// Whether the file was unlinked before the failure occurred.
    pub removed: bool,
    /// Underlying confinement, removal, or directory-sync failure.
    pub source: CacheFileError,
}

/// Scan only the regular files directly beneath one confined managed directory.
#[cfg(feature = "certified-subnet-catalog-host")]
pub fn scan_managed_directory_files(
    cache_root: &Path,
    directory_path: &Path,
    maximum_files: u64,
) -> Result<ManagedDirectoryScan, CacheFileError> {
    #[cfg(not(unix))]
    {
        let _ = (cache_root, directory_path, maximum_files);
        Err(super::unsupported_platform())
    }
    #[cfg(unix)]
    {
        let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
            return Ok(ManagedDirectoryScan::default());
        };
        let probe_path = directory_path.join(".icq-directory-probe");
        let Some(directory) = root.resolve_parent(&probe_path, false)? else {
            return Ok(ManagedDirectoryScan::default());
        };
        let entries = directory
            .entry
            .parent()
            .entries(usize::try_from(maximum_files).unwrap_or(usize::MAX))
            .map_err(|error| capability_error(cache_root, error))?;
        let mut scan = ManagedDirectoryScan::default();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(CapabilityError::EntryLimit { .. }) => {
                    scan.truncated = true;
                    break;
                }
                Err(error) => return Err(capability_error(cache_root, error)),
            };
            let path = directory_path.join(&entry.name);
            if entry.kind == EntryKind::SymbolicLink {
                return Err(confinement_error(
                    cache_root,
                    &path,
                    "managed directory entry is a symbolic link",
                ));
            }
            if entry.kind != EntryKind::File {
                return Err(confinement_error(
                    cache_root,
                    &path,
                    "managed directory entry is not a regular file",
                ));
            }
            let managed = root.resolve_parent(&path, false)?.ok_or_else(|| {
                open_managed_path_error(
                    cache_root,
                    &path,
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "managed directory entry disappeared during discovery",
                    ),
                )
            })?;
            let file = managed.open_regular_file()?.ok_or_else(|| {
                open_managed_path_error(
                    cache_root,
                    &path,
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "managed directory entry disappeared during discovery",
                    ),
                )
            })?;
            let bytes = file
                .metadata()
                .map_err(|source| open_managed_path_error(cache_root, &path, source))?
                .len();
            scan.files.push(ManagedDirectoryFile { path, bytes });
        }
        scan.files.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(scan)
    }
}

/// Remove one confined regular file and synchronize its parent directory.
#[cfg(feature = "certified-subnet-catalog-host")]
pub fn remove_managed_regular_file(
    cache_root: &Path,
    target_path: &Path,
) -> Result<bool, ManagedFileRemovalError> {
    #[cfg(not(unix))]
    {
        let _ = (cache_root, target_path);
        Err(removal_not_completed(super::unsupported_platform()))
    }
    #[cfg(unix)]
    {
        let Some(root) =
            ConfinedCacheRoot::open(cache_root, false).map_err(removal_not_completed)?
        else {
            return Ok(false);
        };
        let Some(target) = root
            .resolve_parent(target_path, false)
            .map_err(removal_not_completed)?
        else {
            return Ok(false);
        };
        let Some(file) = target.open_regular_file().map_err(removal_not_completed)? else {
            return Ok(false);
        };
        drop(file);
        target
            .entry
            .remove_file_and_sync()
            .map_err(|error| ManagedFileRemovalError {
                removed: error.removed,
                source: capability_error(cache_root, error.source),
            })
    }
}

#[cfg(feature = "certified-subnet-catalog-host")]
const fn removal_not_completed(source: CacheFileError) -> ManagedFileRemovalError {
    ManagedFileRemovalError {
        removed: false,
        source,
    }
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
mod tests {
    use super::*;
    use crate::{cache_file::create_managed_parent_directory, test_support::temp_dir};
    use std::fs;
    #[cfg(all(feature = "sns-host", unix))]
    use std::os::unix::fs::{PermissionsExt, symlink};

    #[cfg(feature = "sns-host")]
    #[test]
    fn collection_discovery_requires_a_complete_candidate_and_entry_scan() {
        let root = temp_dir("ic-query-collection-scan-limit");
        let network = root.join("sns/ic");
        for entity in ["b", "a"] {
            super::super::write_managed_text_atomically(
                &root,
                &network.join(entity).join("proposals/full.json"),
                "{}",
            )
            .unwrap();
        }
        let paths =
            collect_managed_collection_files(&root, &network, "proposals", "full.json", 2).unwrap();
        assert_eq!(
            paths,
            vec![
                network.join("a/proposals/full.json"),
                network.join("b/proposals/full.json")
            ]
        );
        super::super::write_managed_text_atomically(
            &root,
            &network.join("c/proposals/full.json"),
            "{}",
        )
        .unwrap();
        assert!(matches!(
            collect_managed_collection_files(&root, &network, "proposals", "full.json", 2),
            Err(CacheFileError::ScanLimitExceeded {
                resource: "collection candidates",
                maximum: 2,
                ..
            })
        ));
        // Nonmatching entities must consume the entry allowance too.
        for entity in 0..14 {
            create_managed_parent_directory(&root, &network.join(entity.to_string()).join("probe"))
                .unwrap();
        }
        assert!(matches!(
            collect_managed_collection_files(&root, &network, "neurons", "full.json", 1),
            Err(CacheFileError::ScanLimitExceeded {
                resource: "directory entries",
                maximum: 16,
                ..
            })
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(all(feature = "sns-host", unix))]
    #[test]
    fn collection_discovery_preserves_confinement_and_permission_errors() {
        let root = temp_dir("ic-query-collection-scan-authority");
        let network = root.join("sns/ic");
        let path = network.join("entity/proposals/full.json");
        super::super::write_managed_text_atomically(&root, &path, "{}").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).unwrap();
        assert!(matches!(
            collect_managed_collection_files(&root, &network, "proposals", "full.json", 2),
            Err(CacheFileError::UnsafeManagedPermissions { .. })
        ));
        fs::remove_file(&path).unwrap();
        symlink(&path, network.join("linked")).unwrap();
        assert!(matches!(
            collect_managed_collection_files(&root, &network, "proposals", "full.json", 2),
            Err(CacheFileError::Confinement { .. })
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(all(unix, feature = "certified-subnet-catalog-host", feature = "sns-host"))]
    #[test]
    fn zero_budget_scans_project_unadmitted_entries_without_following_them() {
        let root = temp_dir("ic-query-zero-scan-budget");
        let outside = temp_dir("ic-query-zero-scan-outside");
        symlink(&outside, root.join("link")).unwrap();
        let inventory = collect_managed_files(&root, 0, |_| true).unwrap();
        assert!(inventory.root_found);
        assert!(inventory.truncated);
        assert_eq!(inventory.paths, Vec::<PathBuf>::new());
        let exact = scan_managed_directory_files(&root, &root, 0).unwrap();
        assert!(exact.truncated);
        assert_eq!(exact.files, Vec::<ManagedDirectoryFile>::new());
        assert!(matches!(
            collect_managed_collection_files(&root, &root, "proposals", "full.json", 0),
            Err(CacheFileError::ScanLimitExceeded {
                resource: "directory entries",
                maximum: 0,
                ..
            })
        ));
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn inventory_bounds_nonmatching_entries_and_directory_depth() {
        let root = temp_dir("ic-query-inventory-entry-bound");
        for index in 0..17 {
            create_managed_parent_directory(&root, &root.join(index.to_string()).join("probe"))
                .unwrap();
        }
        let scan = collect_managed_files(&root, 1, |_| false).unwrap();
        assert!(scan.truncated);
        assert_eq!(scan.paths, Vec::<PathBuf>::new());
        fs::remove_dir_all(&root).unwrap();
        let path = (0..34).fold(root.clone(), |path, _| path.join("nested"));
        create_managed_parent_directory(&root, &path.join("probe")).unwrap();
        let scan = collect_managed_files(&root, 100, |_| false).unwrap();
        assert!(scan.truncated);
        fs::remove_dir_all(root).unwrap();
    }
}
