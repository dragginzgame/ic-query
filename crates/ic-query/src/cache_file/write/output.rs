//! Module: cache_file::write::output
//!
//! Responsibility: write explicit refresh output files.
//! Does not own: cache replacement, refresh locking, or JSON serialization.
//! Boundary: rejects managed-path aliases before writing/syncing explicit exports.

use super::path::create_output_parent_directory;
use crate::cache_file::CacheFileError;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::{
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

pub fn validate_output_path(
    output_path: &Path,
    managed_paths: &[&Path],
) -> Result<(), CacheFileError> {
    let resolved_output =
        resolve_output_path(output_path, 0).map_err(|source| output_error(output_path, source))?;
    let output_metadata =
        optional_metadata(output_path).map_err(|source| output_error(output_path, source))?;
    for &managed_path in managed_paths {
        let resolved_managed = resolve_output_path(managed_path, 0)
            .map_err(|source| output_error(output_path, source))?;
        let managed_metadata =
            optional_metadata(managed_path).map_err(|source| output_error(output_path, source))?;
        if resolved_output == resolved_managed
            || output_metadata
                .as_ref()
                .zip(managed_metadata.as_ref())
                .is_some_and(|(output, managed)| same_file(output, managed))
        {
            return Err(output_alias_error(output_path, managed_path));
        }
    }
    Ok(())
}

pub fn write_text_output(
    output_path: &Path,
    contents: &str,
    managed_paths: &[&Path],
) -> Result<(), CacheFileError> {
    validate_output_path(output_path, managed_paths)?;
    create_output_parent_directory(output_path)?;
    // Open without truncation so an alias introduced since validation cannot erase a cache.
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(output_path)
        .map_err(|source| output_error(output_path, source))?;
    validate_output_path(output_path, managed_paths)?;
    let output_metadata = output
        .metadata()
        .map_err(|source| output_error(output_path, source))?;
    for &managed_path in managed_paths {
        if optional_metadata(managed_path)
            .map_err(|source| output_error(output_path, source))?
            .is_some_and(|managed| same_file(&output_metadata, &managed))
        {
            return Err(output_alias_error(output_path, managed_path));
        }
    }
    output
        .set_len(0)
        .map_err(|source| output_error(output_path, source))?;
    output
        .write_all(contents.as_bytes())
        .map_err(|source| CacheFileError::WriteOutput {
            path: output_path.to_path_buf(),
            source,
        })?;
    output
        .sync_all()
        .map_err(|source| CacheFileError::SyncOutput {
            path: output_path.to_path_buf(),
            source,
        })
}

fn optional_metadata(path: &Path) -> io::Result<Option<fs::Metadata>> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        left.dev() == right.dev() && left.ino() == right.ino()
    }
    #[cfg(not(unix))]
    {
        let _ = (left, right);
        false
    }
}

fn resolve_output_path(path: &Path, symlink_depth: usize) -> io::Result<PathBuf> {
    let path = if path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        path
    };
    match fs::canonicalize(path) {
        Ok(resolved) => return Ok(resolved),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    // Also resolve dangling links and missing targets before directories or locks exist.
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_symlink()) {
        if symlink_depth >= 64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many output symlinks",
            ));
        }
        let target = fs::read_link(path)?;
        let target = if target.is_absolute() {
            target
        } else {
            path.parent().unwrap_or_else(|| Path::new(".")).join(target)
        };
        return resolve_output_path(&target, symlink_depth + 1);
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output path has no parent"))?;
    let mut resolved = resolve_output_path(parent, symlink_depth)?;
    match path.components().next_back() {
        Some(Component::Normal(name)) => resolved.push(name),
        Some(Component::ParentDir) => {
            resolved.pop();
        }
        Some(Component::CurDir) => {}
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid output path",
            ));
        }
    }
    Ok(resolved)
}

fn output_alias_error(output_path: &Path, managed_path: &Path) -> CacheFileError {
    CacheFileError::OutputAliasesManagedPath {
        output_path: output_path.to_path_buf(),
        managed_path: managed_path.to_path_buf(),
    }
}

fn output_error(path: &Path, source: io::Error) -> CacheFileError {
    CacheFileError::WriteOutput {
        path: path.to_path_buf(),
        source,
    }
}
