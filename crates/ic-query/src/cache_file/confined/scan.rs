//! Module: cache_file::confined::scan
//!
//! Responsibility: bounded confined-cache discovery and regular-file removal.
//! Does not own: cache schemas, path confinement, or atomic publication.
//! Boundary: traverses capability-rooted directories without following symbolic links.

#[cfg(feature = "sns-host")]
use super::validate_managed_file_mode;
use super::{CacheFileError, ConfinedCacheRoot, confinement_error, open_managed_path_error};
#[cfg(feature = "host")]
use super::{open_directory_component, validate_managed_directory_mode};
#[cfg(any(feature = "certified-subnet-catalog-host", feature = "host"))]
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
    /// Whether discovery stopped at the caller's selected-file limit.
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
    let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
        return Ok(ManagedDirectoryScan::default());
    };
    let probe_path = directory_path.join(".icq-directory-probe");
    let Some(directory) = root.resolve_parent(&probe_path, false)? else {
        return Ok(ManagedDirectoryScan::default());
    };
    let entries = directory
        .parent
        .entries()
        .map_err(|source| open_managed_path_error(cache_root, directory_path, source))?;
    let mut scan = ManagedDirectoryScan::default();
    for entry in entries {
        if u64::try_from(scan.files.len()).map_or(true, |count| count == maximum_files) {
            scan.truncated = true;
            break;
        }
        let entry =
            entry.map_err(|source| open_managed_path_error(cache_root, directory_path, source))?;
        let name = entry.file_name();
        let path = directory_path.join(&name);
        let file_type = entry
            .file_type()
            .map_err(|source| open_managed_path_error(cache_root, &path, source))?;
        if file_type.is_symlink() {
            return Err(confinement_error(
                cache_root,
                &path,
                "managed directory entry is a symbolic link",
            ));
        }
        if !file_type.is_file() {
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

/// Remove one confined regular file and synchronize its parent directory.
#[cfg(feature = "certified-subnet-catalog-host")]
pub fn remove_managed_regular_file(
    cache_root: &Path,
    target_path: &Path,
) -> Result<bool, ManagedFileRemovalError> {
    let Some(root) = ConfinedCacheRoot::open(cache_root, false).map_err(removal_not_completed)?
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
    match target.remove_file() {
        Ok(()) => {}
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(source) => {
            return Err(removal_not_completed(CacheFileError::RemoveManagedFile {
                path: target_path.to_path_buf(),
                source,
            }));
        }
    }
    target
        .sync_parent()
        .map_err(|source| ManagedFileRemovalError {
            removed: true,
            source,
        })?;
    Ok(true)
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
    let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
        return Ok(ManagedFileScan::default());
    };
    let root_dir = root
        .dir
        .try_clone()
        .map_err(|source| open_managed_path_error(cache_root, cache_root, source))?;
    let mut directories = vec![(root_dir, root.display_root.clone())];
    let mut scan = ManagedFileScan {
        root_found: true,
        ..ManagedFileScan::default()
    };
    while let Some((directory, display_directory)) = directories.pop() {
        let entries = directory
            .entries()
            .map_err(|source| open_managed_path_error(cache_root, &display_directory, source))?;
        for entry in entries {
            let entry = entry.map_err(|source| {
                open_managed_path_error(cache_root, &display_directory, source)
            })?;
            let name = entry.file_name();
            let path = display_directory.join(&name);
            let file_type = entry
                .file_type()
                .map_err(|source| open_managed_path_error(cache_root, &path, source))?;
            if file_type.is_symlink() {
                return Err(confinement_error(
                    cache_root,
                    &path,
                    "managed cache entry is a symbolic link",
                ));
            }
            if file_type.is_dir() {
                let child = open_directory_component(&directory, &name, cache_root, &path)?
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
                validate_managed_directory_mode(&path, &child)?;
                directories.push((child, path));
                continue;
            }
            if !file_type.is_file() {
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

/// Discover canonical collection files beneath one confined network directory.
#[cfg(feature = "sns-host")]
pub fn collect_managed_collection_files(
    cache_root: &Path,
    network_dir: &Path,
    collection: &str,
    file_name: &str,
) -> Result<Vec<PathBuf>, CacheFileError> {
    let Some(root) = ConfinedCacheRoot::open(cache_root, false)? else {
        return Ok(Vec::new());
    };
    let probe_path = network_dir.join(".icq-directory-probe");
    let Some(network) = root.resolve_parent(&probe_path, false)? else {
        return Ok(Vec::new());
    };
    let entries = network
        .parent
        .entries()
        .map_err(|source| open_managed_path_error(cache_root, network_dir, source))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|source| open_managed_path_error(cache_root, network_dir, source))?;
        let file_type = entry
            .file_type()
            .map_err(|source| open_managed_path_error(cache_root, network_dir, source))?;
        let entity_path = network_dir.join(entry.file_name());
        if file_type.is_symlink() {
            return Err(confinement_error(
                cache_root,
                &entity_path,
                "managed collection entity is a symbolic link",
            ));
        }
        if !file_type.is_dir() {
            continue;
        }
        let candidate = entity_path.join(collection).join(file_name);
        let Some(candidate_path) = root.resolve_parent(&candidate, false)? else {
            continue;
        };
        if let Some(file) = candidate_path.open_regular_file()? {
            validate_managed_file_mode(&candidate, &file)?;
            paths.push(candidate);
        }
    }
    paths.sort();
    Ok(paths)
}
