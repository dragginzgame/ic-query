use crate::cache_file::CacheFileError;
use std::{fs, io, path::Path};

fn create_directory(path: &Path) -> Result<(), CacheFileError> {
    fs::create_dir_all(path).map_err(|source| CacheFileError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

pub fn create_parent_directory(target_path: &Path) -> Result<(), CacheFileError> {
    create_directory(target_directory(target_path)?)
}

pub(super) fn sync_directory(path: &Path) -> Result<(), CacheFileError> {
    fs::File::open(path)
        .and_then(|dir| dir.sync_all())
        .map_err(|source| CacheFileError::SyncDirectory {
            path: path.to_path_buf(),
            source,
        })
}

pub(super) fn target_directory(target_path: &Path) -> Result<&Path, CacheFileError> {
    let parent = target_path
        .parent()
        .ok_or_else(|| invalid_target_path_error(target_path))?;
    if parent.as_os_str().is_empty() {
        Ok(Path::new("."))
    } else {
        Ok(parent)
    }
}

fn invalid_target_path_error(target_path: &Path) -> CacheFileError {
    CacheFileError::WriteTemp {
        path: target_path.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::InvalidInput,
            "cache target path must have a parent directory",
        ),
    }
}
