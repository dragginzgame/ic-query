//! Module: cache_file::write::output
//!
//! Responsibility: write explicit refresh output files.
//! Does not own: cache replacement, refresh locking, or JSON serialization.
//! Boundary: rejects managed-path aliases before writing/syncing explicit exports.

use super::path::create_output_parent_directory;
use crate::cache_file::CacheFileError;
use ic_host_fs::path::canonicalize_allow_missing_with_symlink_limit;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

const OUTPUT_SYMLINK_LIMIT: usize = 64;

pub fn validate_output_path(
    output_path: &Path,
    managed_paths: &[&Path],
) -> Result<(), CacheFileError> {
    let base = if output_path.is_relative() || managed_paths.iter().any(|path| path.is_relative()) {
        fs::canonicalize(".").map_err(|source| output_error(output_path, source))?
    } else {
        PathBuf::new()
    };
    let resolved_output =
        canonicalize_allow_missing_with_symlink_limit(output_path, &base, OUTPUT_SYMLINK_LIMIT)
            .map_err(|source| output_error(output_path, source))?;
    let output_metadata =
        optional_metadata(output_path).map_err(|source| output_error(output_path, source))?;
    for &managed_path in managed_paths {
        let resolved_managed = canonicalize_allow_missing_with_symlink_limit(
            managed_path,
            &base,
            OUTPUT_SYMLINK_LIMIT,
        )
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
