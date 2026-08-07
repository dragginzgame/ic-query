//! Module: cache_file::confined
//!
//! Responsibility: capability-rooted managed cache path resolution and file IO.
//! Does not own: JSON schemas, refresh policy, or caller-selected export paths.
//! Boundary: rejects traversal, symlinks, nonregular files, and unsafe managed modes.

use super::CacheFileError;
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::{
    ambient_authority,
    fs::{Dir, DirBuilder, OpenOptions},
};
use std::{
    ffi::{OsStr, OsString},
    io,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(any(feature = "subnet-catalog-host", test))]
use std::io::Write;

#[cfg(unix)]
use cap_std::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

mod read;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "host",
    feature = "sns-host"
))]
mod scan;

#[cfg(feature = "host")]
pub use read::open_managed_file;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "icrc-host",
    feature = "sns-host",
    test
))]
pub use read::read_bounded_managed_file;
pub use read::{BoundedManagedFileReadError, managed_file_exists, read_managed_text};
#[cfg(feature = "sns-host")]
pub use scan::collect_managed_collection_files;
#[cfg(feature = "host")]
pub use scan::collect_managed_files;
#[cfg(feature = "certified-subnet-catalog-host")]
pub use scan::{ManagedDirectoryFile, remove_managed_regular_file, scan_managed_directory_files};

const MANAGED_DIRECTORY_MODE: u32 = 0o700;
const MANAGED_FILE_MODE: u32 = 0o600;
const OWNER_ONLY_DIRECTORY_MODE: &str = "no group or other access";
const OWNER_READ_WRITE_FILE_MODE: &str = "mode 0o600";

static ATOMIC_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Create and validate the managed parent directory beneath `cache_root`.
pub fn create_managed_parent_directory(
    cache_root: &Path,
    target_path: &Path,
) -> Result<(), CacheFileError> {
    let root = ConfinedCacheRoot::open(cache_root, true)?.ok_or_else(|| {
        open_managed_path_error(
            cache_root,
            target_path,
            io::Error::new(io::ErrorKind::NotFound, "cache root was not created"),
        )
    })?;
    root.resolve_parent(target_path, true)?.ok_or_else(|| {
        open_managed_path_error(
            cache_root,
            target_path,
            io::Error::new(io::ErrorKind::NotFound, "cache parent was not created"),
        )
    })?;
    Ok(())
}

/// Atomically publish UTF-8 text through a confined same-directory temporary file.
#[cfg(any(feature = "subnet-catalog-host", test))]
pub fn write_managed_text_atomically(
    cache_root: &Path,
    target_path: &Path,
    contents: &str,
) -> Result<(), CacheFileError> {
    write_managed_file_atomically(cache_root, target_path, |file| {
        file.write_all(contents.as_bytes())
    })
}

/// Atomically publish a streamed managed file through a confined same-directory temporary file.
pub fn write_managed_file_atomically(
    cache_root: &Path,
    target_path: &Path,
    write: impl FnOnce(&mut cap_std::fs::File) -> io::Result<()>,
) -> Result<(), CacheFileError> {
    let root = ConfinedCacheRoot::open(cache_root, true)?.ok_or_else(|| {
        open_managed_path_error(
            cache_root,
            target_path,
            io::Error::new(io::ErrorKind::NotFound, "cache root was not created"),
        )
    })?;
    let target = root.resolve_parent(target_path, true)?.ok_or_else(|| {
        open_managed_path_error(
            cache_root,
            target_path,
            io::Error::new(io::ErrorKind::NotFound, "cache parent was not created"),
        )
    })?;
    target.validate_existing_target()?;
    let temp_name = atomic_temp_name(target.file_name());
    let temp_path = target.display_parent.join(&temp_name);
    let write_result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        #[cfg(unix)]
        options.mode(MANAGED_FILE_MODE);
        let mut temp = target
            .parent
            .open_with(&temp_name, &options)
            .map_err(|source| CacheFileError::WriteTemp {
                path: temp_path.clone(),
                source,
            })?;
        validate_managed_file_mode(&temp_path, &temp)?;
        write(&mut temp).map_err(|source| CacheFileError::WriteTemp {
            path: temp_path.clone(),
            source,
        })?;
        temp.sync_all().map_err(|source| CacheFileError::SyncTemp {
            path: temp_path.clone(),
            source,
        })
    })();
    if let Err(error) = write_result {
        let _ = target.parent.remove_file(&temp_name);
        return Err(error);
    }
    if let Err(source) = target
        .parent
        .rename(&temp_name, &target.parent, target.file_name())
    {
        let _ = target.parent.remove_file(&temp_name);
        return Err(CacheFileError::Replace {
            temp_path,
            target_path: target_path.to_path_buf(),
            source,
        });
    }
    sync_directory(&target.parent, &target.display_parent)
}

pub(super) fn managed_path_for_create(
    cache_root: &Path,
    target_path: &Path,
) -> Result<ConfinedManagedPath, CacheFileError> {
    let root = ConfinedCacheRoot::open(cache_root, true)?.ok_or_else(|| {
        open_managed_path_error(
            cache_root,
            target_path,
            io::Error::new(io::ErrorKind::NotFound, "cache root was not created"),
        )
    })?;
    root.resolve_parent(target_path, true)?.ok_or_else(|| {
        open_managed_path_error(
            cache_root,
            target_path,
            io::Error::new(io::ErrorKind::NotFound, "cache parent was not created"),
        )
    })
}

///
/// ConfinedCacheRoot
///
/// Open directory capability that anchors every managed cache operation.
///

pub(super) struct ConfinedCacheRoot {
    display_root: PathBuf,
    absolute_root: PathBuf,
    dir: Dir,
}

impl ConfinedCacheRoot {
    pub(super) fn open(cache_root: &Path, create: bool) -> Result<Option<Self>, CacheFileError> {
        #[cfg(not(unix))]
        {
            let _ = (cache_root, create);
            return Err(CacheFileError::UnsupportedConfinementPlatform {
                platform: std::env::consts::OS,
            });
        }
        #[cfg(unix)]
        {
            let absolute_root = absolute_managed_path(cache_root, cache_root)?;
            let mut dir = Dir::open_ambient_dir(Path::new("/"), ambient_authority())
                .map_err(|source| open_managed_path_error(cache_root, cache_root, source))?;
            for component in absolute_root.components() {
                let Component::Normal(name) = component else {
                    continue;
                };
                dir = match open_directory_component(&dir, name, cache_root, cache_root)? {
                    Some(next) => next,
                    None if create => {
                        create_directory_component(&dir, name, cache_root, cache_root)?;
                        open_directory_component(&dir, name, cache_root, cache_root)?.ok_or_else(
                            || {
                                open_managed_path_error(
                                    cache_root,
                                    cache_root,
                                    io::Error::new(
                                        io::ErrorKind::NotFound,
                                        "created cache root component disappeared",
                                    ),
                                )
                            },
                        )?
                    }
                    None => return Ok(None),
                };
            }
            validate_managed_directory_mode(cache_root, &dir)?;
            Ok(Some(Self {
                display_root: cache_root.to_path_buf(),
                absolute_root,
                dir,
            }))
        }
    }

    pub(super) fn resolve_parent(
        &self,
        target_path: &Path,
        create: bool,
    ) -> Result<Option<ConfinedManagedPath>, CacheFileError> {
        let absolute_target = absolute_managed_path(&self.display_root, target_path)?;
        let relative = absolute_target
            .strip_prefix(&self.absolute_root)
            .map_err(|_| {
                confinement_error(
                    &self.display_root,
                    target_path,
                    "path is outside the cache root",
                )
            })?;
        let file_name = relative.file_name().ok_or_else(|| {
            confinement_error(
                &self.display_root,
                target_path,
                "managed path must name a file beneath the cache root",
            )
        })?;
        let relative_parent = relative.parent().unwrap_or_else(|| Path::new(""));
        let mut parent = self
            .dir
            .try_clone()
            .map_err(|source| open_managed_path_error(&self.display_root, target_path, source))?;
        let mut display_parent = self.display_root.clone();
        for component in relative_parent.components() {
            let Component::Normal(name) = component else {
                return Err(confinement_error(
                    &self.display_root,
                    target_path,
                    "managed relative path contains a non-normal component",
                ));
            };
            display_parent.push(name);
            parent =
                match open_directory_component(&parent, name, &self.display_root, &display_parent)?
                {
                    Some(next) => next,
                    None if create => {
                        create_directory_component(
                            &parent,
                            name,
                            &self.display_root,
                            &display_parent,
                        )?;
                        open_directory_component(
                            &parent,
                            name,
                            &self.display_root,
                            &display_parent,
                        )?
                        .ok_or_else(|| {
                            open_managed_path_error(
                                &self.display_root,
                                &display_parent,
                                io::Error::new(
                                    io::ErrorKind::NotFound,
                                    "created managed directory disappeared",
                                ),
                            )
                        })?
                    }
                    None => return Ok(None),
                };
            validate_managed_directory_mode(&display_parent, &parent)?;
        }
        Ok(Some(ConfinedManagedPath {
            root: self.display_root.clone(),
            parent,
            display_parent,
            file_name: file_name.to_os_string(),
            display_path: target_path.to_path_buf(),
        }))
    }
}

///
/// ConfinedManagedPath
///
/// Resolved parent capability and final component for one managed file.
///

pub(super) struct ConfinedManagedPath {
    root: PathBuf,
    parent: Dir,
    display_parent: PathBuf,
    file_name: OsString,
    display_path: PathBuf,
}

impl ConfinedManagedPath {
    pub(super) fn file_name(&self) -> &OsStr {
        &self.file_name
    }

    pub(super) fn open_regular_file(&self) -> Result<Option<cap_std::fs::File>, CacheFileError> {
        match self.parent.symlink_metadata(&self.file_name) {
            Ok(metadata) if metadata.file_type().is_symlink() => Err(confinement_error(
                &self.root,
                &self.display_path,
                "managed file is a symbolic link",
            )),
            Ok(metadata) if !metadata.is_file() => Err(confinement_error(
                &self.root,
                &self.display_path,
                "managed path is not a regular file",
            )),
            Ok(_) => {
                let mut options = OpenOptions::new();
                options.read(true).follow(FollowSymlinks::No);
                let file = self
                    .parent
                    .open_with(&self.file_name, &options)
                    .map_err(|source| {
                        open_managed_path_error(&self.root, &self.display_path, source)
                    })?;
                let metadata = file.metadata().map_err(|source| {
                    open_managed_path_error(&self.root, &self.display_path, source)
                })?;
                if !metadata.is_file() {
                    return Err(confinement_error(
                        &self.root,
                        &self.display_path,
                        "opened managed path is not a regular file",
                    ));
                }
                validate_managed_file_mode(&self.display_path, &file)?;
                Ok(Some(file))
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(open_managed_path_error(
                &self.root,
                &self.display_path,
                source,
            )),
        }
    }

    pub(super) fn create_new_file(&self) -> Result<cap_std::fs::File, io::Error> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        #[cfg(unix)]
        options.mode(MANAGED_FILE_MODE);
        let file = self.parent.open_with(&self.file_name, &options)?;
        validate_managed_file_mode(&self.display_path, &file).map_err(io::Error::other)?;
        Ok(file)
    }

    pub(super) fn remove_file(&self) -> Result<(), io::Error> {
        self.parent.remove_file(&self.file_name)
    }

    pub(super) fn sync_parent(&self) -> Result<(), CacheFileError> {
        sync_directory(&self.parent, &self.display_parent)
    }

    pub(super) fn display_path(&self) -> &Path {
        &self.display_path
    }

    fn validate_existing_target(&self) -> Result<(), CacheFileError> {
        drop(self.open_regular_file()?);
        Ok(())
    }
}

fn open_directory_component(
    parent: &Dir,
    name: &OsStr,
    root: &Path,
    display_path: &Path,
) -> Result<Option<Dir>, CacheFileError> {
    match parent.open_dir_nofollow(name) {
        Ok(dir) => Ok(Some(dir)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => match parent.symlink_metadata(name) {
            Ok(metadata) if metadata.file_type().is_symlink() => Err(confinement_error(
                root,
                display_path,
                "managed directory component is a symbolic link",
            )),
            Ok(metadata) if !metadata.is_dir() => Err(confinement_error(
                root,
                display_path,
                "managed directory component is not a directory",
            )),
            _ => Err(open_managed_path_error(root, display_path, source)),
        },
    }
}

fn create_directory_component(
    parent: &Dir,
    name: &OsStr,
    root: &Path,
    display_path: &Path,
) -> Result<(), CacheFileError> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    builder.mode(MANAGED_DIRECTORY_MODE);
    parent
        .create_dir_with(name, &builder)
        .map_err(|source| CacheFileError::CreateDirectory {
            path: display_path.to_path_buf(),
            source,
        })?;
    let dir = open_directory_component(parent, name, root, display_path)?.ok_or_else(|| {
        open_managed_path_error(
            root,
            display_path,
            io::Error::new(io::ErrorKind::NotFound, "created directory disappeared"),
        )
    })?;
    validate_managed_directory_mode(display_path, &dir)?;
    sync_directory(parent, display_path.parent().unwrap_or(display_path))
}

fn absolute_managed_path(root: &Path, path: &Path) -> Result<PathBuf, CacheFileError> {
    for component in path.components() {
        if matches!(component, Component::ParentDir | Component::Prefix(_)) {
            return Err(confinement_error(
                root,
                path,
                "parent traversal and platform prefixes are unsupported",
            ));
        }
    }
    std::path::absolute(path).map_err(|source| open_managed_path_error(root, path, source))
}

fn validate_managed_directory_mode(path: &Path, dir: &Dir) -> Result<(), CacheFileError> {
    #[cfg(unix)]
    {
        let mode = dir
            .dir_metadata()
            .map_err(|source| open_managed_path_error(path, path, source))?
            .permissions()
            .mode()
            & 0o777;
        if mode & 0o077 != 0 {
            return Err(CacheFileError::UnsafeManagedPermissions {
                path: path.to_path_buf(),
                actual_mode: mode,
                required_mode: OWNER_ONLY_DIRECTORY_MODE,
            });
        }
    }
    Ok(())
}

fn validate_managed_file_mode(path: &Path, file: &cap_std::fs::File) -> Result<(), CacheFileError> {
    #[cfg(unix)]
    {
        let mode = file
            .metadata()
            .map_err(|source| open_managed_path_error(path, path, source))?
            .permissions()
            .mode()
            & 0o777;
        if mode != MANAGED_FILE_MODE {
            return Err(CacheFileError::UnsafeManagedPermissions {
                path: path.to_path_buf(),
                actual_mode: mode,
                required_mode: OWNER_READ_WRITE_FILE_MODE,
            });
        }
    }
    Ok(())
}

fn sync_directory(dir: &Dir, display_path: &Path) -> Result<(), CacheFileError> {
    dir.open(Path::new("."))
        .and_then(|directory| directory.sync_all())
        .map_err(|source| CacheFileError::SyncDirectory {
            path: display_path.to_path_buf(),
            source,
        })
}

fn atomic_temp_name(target_file: &OsStr) -> OsString {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = ATOMIC_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut name = target_file.to_os_string();
    name.push(format!(
        ".tmp.{}.{}.{}",
        std::process::id(),
        now_nanos,
        counter
    ));
    name
}

fn confinement_error(root: &Path, path: &Path, reason: impl Into<String>) -> CacheFileError {
    CacheFileError::Confinement {
        root: root.to_path_buf(),
        path: path.to_path_buf(),
        reason: reason.into(),
    }
}

fn open_managed_path_error(root: &Path, path: &Path, source: io::Error) -> CacheFileError {
    CacheFileError::OpenManagedPath {
        root: root.to_path_buf(),
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
