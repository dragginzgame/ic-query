//! Module: cache_file::confined
//!
//! Responsibility: capability-rooted managed cache path resolution and file IO.
//! Does not own: JSON schemas, refresh policy, or caller-selected export paths.
//! Boundary: rejects traversal, symlinks, nonregular files, and unsafe managed modes.

use super::CacheFileError;
use std::{
    fs::File,
    io,
    path::{Component, Path, PathBuf},
};

#[cfg(unix)]
use ic_host_fs::{
    capability::{CapabilityError, Directory, DirectoryOptions, FileEntry, Operation, Refusal},
    durable::{NamedWriteError, PublicationMode, WriteOptions, write_at_validated_with},
    read::{OpenedEntry, open_entry_beneath},
};
#[cfg(unix)]
use std::os::{fd::AsFd, unix::fs::PermissionsExt};

#[cfg(any(feature = "subnet-catalog-host", test))]
use std::io::Write;

mod read;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "host",
    feature = "sns-host"
))]
mod scan;

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use read::ManagedReadBudget;
#[cfg(any(feature = "host", test))]
pub use read::open_managed_file;
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "subnet-catalog-host",
    feature = "dashboard-host",
    feature = "nns-topology-host",
    feature = "icrc-host",
    feature = "sns-host",
    test
))]
pub use read::read_bounded_managed_file;
#[cfg(feature = "host")]
pub use read::read_bounded_stream;
pub use read::{BoundedManagedFileReadError, managed_file_exists};
#[cfg(feature = "sns-host")]
pub use scan::collect_managed_collection_files;
#[cfg(feature = "host")]
pub use scan::collect_managed_files;
#[cfg(feature = "certified-subnet-catalog-host")]
pub use scan::{ManagedDirectoryFile, remove_managed_regular_file, scan_managed_directory_files};

const MANAGED_DIRECTORY_MODE: u32 = 0o700;
const MANAGED_FILE_MODE: u32 = 0o600;
const OWNER_ONLY_WRITE_MODE: &str = "no group or other write access";

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
pub fn write_managed_file_atomically<E: Into<io::Error>>(
    cache_root: &Path,
    target_path: &Path,
    write: impl FnOnce(&mut std::fs::File) -> Result<(), E>,
) -> Result<(), CacheFileError> {
    let target = managed_path_for_create(cache_root, target_path)?;
    target.write_atomically(write)
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
    #[cfg(unix)]
    dir: Directory,
}

impl ConfinedCacheRoot {
    pub(super) fn open(cache_root: &Path, create: bool) -> Result<Option<Self>, CacheFileError> {
        #[cfg(not(unix))]
        {
            let _ = (cache_root, create);
            Err(unsupported_platform())
        }
        #[cfg(unix)]
        {
            let absolute_root = absolute_managed_path(cache_root, cache_root)?;
            let dir = Directory::open_root(cache_root, directory_options(create))
                .map_err(|error| capability_error(cache_root, error))?;
            Ok(dir.map(|dir| Self {
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
        #[cfg(not(unix))]
        {
            let _ = (target_path, create);
            Err(unsupported_platform())
        }
        #[cfg(unix)]
        {
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
            if relative.file_name().is_none() {
                return Err(confinement_error(
                    &self.display_root,
                    target_path,
                    "managed path must name a file beneath the cache root",
                ));
            }
            let entry = self
                .dir
                .resolve_parent(relative, directory_options(create))
                .map_err(|error| capability_error(&self.display_root, error))?;
            Ok(entry.map(|entry| ConfinedManagedPath {
                root: self.display_root.clone(),
                entry,
                display_path: target_path.to_path_buf(),
            }))
        }
    }
}

///
/// ConfinedManagedPath
///
/// Host-held parent and leaf with Query's cache identity and error projection.
///

pub(super) struct ConfinedManagedPath {
    root: PathBuf,
    #[cfg(unix)]
    entry: FileEntry,
    display_path: PathBuf,
}

impl ConfinedManagedPath {
    fn write_atomically<E: Into<io::Error>>(
        &self,
        write: impl FnOnce(&mut File) -> Result<(), E>,
    ) -> Result<(), CacheFileError> {
        drop(self.open_regular_file()?);
        #[cfg(unix)]
        {
            let parent = self.entry.parent().open_syncable().map_err(|error| {
                let source = match error {
                    CapabilityError::Io { source, .. } => source,
                    error => io::Error::other(error),
                };
                managed_publication_error(
                    &self.display_path,
                    NamedWriteError::<io::Error>::BeforePublication {
                        source,
                        cleanup_error: None,
                    },
                )
            })?;
            write_at_validated_with(
                parent.as_fd(),
                self.entry.file_name(),
                WriteOptions {
                    mode: PublicationMode::Replace,
                    permissions: MANAGED_FILE_MODE,
                },
                write,
                |()| Ok(()),
            )
            .map_err(|error| managed_publication_error(&self.display_path, error))
        }
        #[cfg(not(unix))]
        {
            let _ = write;
            Err(unsupported_platform())
        }
    }

    pub(super) fn open_regular_file(&self) -> Result<Option<File>, CacheFileError> {
        #[cfg(unix)]
        {
            let file = self
                .entry
                .open_regular_file()
                .map_err(|error| capability_error(&self.root, error))?;
            if let Some(file) = &file {
                validate_managed_file_mode(&self.display_path, file)?;
            }
            Ok(file)
        }
        #[cfg(not(unix))]
        Err(unsupported_platform())
    }

    pub(super) fn open_coordination_file(&self) -> Result<File, CacheFileError> {
        #[cfg(unix)]
        {
            let file = match self.entry.create_new_file(MANAGED_FILE_MODE) {
                Ok(file) => file,
                Err(CapabilityError::Io { source, .. })
                    if source.kind() == io::ErrorKind::AlreadyExists =>
                {
                    match open_entry_beneath(
                        self.entry.parent().as_fd(),
                        Path::new(self.entry.file_name()),
                    )
                    .map_err(|source| CacheFileError::CreateRefreshLock {
                        path: self.display_path.clone(),
                        source: io::Error::other(source),
                    })? {
                        OpenedEntry::RegularFile(file) => file,
                        OpenedEntry::Directory(_) => {
                            return Err(confinement_error(
                                &self.root,
                                &self.display_path,
                                "coordination file is not a regular file",
                            ));
                        }
                    }
                }
                Err(CapabilityError::Io { source, .. }) => {
                    return Err(CacheFileError::CreateRefreshLock {
                        path: self.display_path.clone(),
                        source,
                    });
                }
                Err(error) => return Err(capability_error(&self.root, error)),
            };
            validate_managed_file_mode(&self.display_path, &file)?;
            Ok(file)
        }
        #[cfg(not(unix))]
        Err(unsupported_platform())
    }
}

#[cfg(unix)]
const fn directory_options(create: bool) -> DirectoryOptions {
    DirectoryOptions {
        create_permissions: if create {
            Some(MANAGED_DIRECTORY_MODE)
        } else {
            None
        },
        forbidden_permissions: 0o022,
    }
}

#[cfg(unix)]
fn capability_error(root: &Path, error: CapabilityError) -> CacheFileError {
    match error {
        CapabilityError::Io {
            operation: Operation::CreateDirectory,
            path,
            source,
        } => CacheFileError::CreateDirectory { path, source },
        CapabilityError::Io {
            operation: Operation::SyncDirectory,
            path,
            source,
        } => CacheFileError::SyncDirectory { path, source },
        CapabilityError::Io {
            operation: Operation::RemoveFile,
            path,
            source,
        } => CacheFileError::RemoveManagedFile { path, source },
        CapabilityError::Io { path, source, .. } => open_managed_path_error(root, &path, source),
        CapabilityError::Permissions { path, actual, .. } => {
            CacheFileError::UnsafeManagedPermissions {
                path,
                actual_mode: actual,
                required_mode: OWNER_ONLY_WRITE_MODE,
            }
        }
        CapabilityError::Refused { path, reason } => confinement_error(
            root,
            &path,
            match reason {
                Refusal::InvalidPath => "managed relative path contains a non-normal component",
                Refusal::SymbolicLink => "managed path is a symbolic link",
                Refusal::NotDirectory => "managed directory component is not a directory",
                Refusal::NotRegularFile => "managed path is not a regular file",
            },
        ),
        CapabilityError::EntryLimit { path, maximum } => CacheFileError::ScanLimitExceeded {
            path,
            resource: "directory entries",
            maximum: maximum as u64,
        },
    }
}

#[cfg(not(unix))]
const fn unsupported_platform() -> CacheFileError {
    CacheFileError::UnsupportedConfinementPlatform {
        platform: std::env::consts::OS,
    }
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

fn validate_managed_file_mode(path: &Path, file: &File) -> Result<(), CacheFileError> {
    #[cfg(unix)]
    {
        let mode = file
            .metadata()
            .map_err(|source| open_managed_path_error(path, path, source))?
            .permissions()
            .mode()
            & 0o777;
        validate_managed_mode(path, mode)?;
    }
    Ok(())
}

#[cfg(unix)]
fn validate_managed_mode(path: &Path, mode: u32) -> Result<(), CacheFileError> {
    if mode & 0o022 != 0 {
        return Err(CacheFileError::UnsafeManagedPermissions {
            path: path.to_path_buf(),
            actual_mode: mode,
            required_mode: OWNER_ONLY_WRITE_MODE,
        });
    }
    Ok(())
}

#[cfg(unix)]
fn managed_publication_error<E: Into<io::Error>>(
    path: &Path,
    error: NamedWriteError<E>,
) -> CacheFileError {
    let (source, published, cleanup_error) = match error {
        NamedWriteError::Producer {
            source,
            cleanup_error,
        } => (source.into(), false, cleanup_error),
        NamedWriteError::BeforePublication {
            source,
            cleanup_error,
        } => (source, false, cleanup_error),
        NamedWriteError::AfterPublication { source } => (source, true, None),
    };
    CacheFileError::PublishManagedFile {
        path: path.to_path_buf(),
        published,
        source,
        cleanup_error,
    }
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
