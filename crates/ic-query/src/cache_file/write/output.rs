//! Caller-selected exports that must preserve managed cache and lock paths.
//!
//! Host owns path and inode checks, in-place writing and synchronization. Query
//! selects protected paths and the symlink limit, and preserves its typed errors.

use crate::cache_file::CacheFileError;
use ic_host_fs::durable::{NonAliasingWriteError, validate_non_aliasing, write_non_aliasing};
use std::path::Path;

const OUTPUT_SYMLINK_LIMIT: usize = 64;

pub fn validate_output_path(
    output_path: &Path,
    managed_paths: &[&Path],
) -> Result<(), CacheFileError> {
    validate_non_aliasing(output_path, managed_paths, OUTPUT_SYMLINK_LIMIT).map_err(export_error)
}

pub fn write_text_output(
    output_path: &Path,
    contents: &str,
    managed_paths: &[&Path],
) -> Result<(), CacheFileError> {
    write_non_aliasing(
        output_path,
        contents.as_bytes(),
        managed_paths,
        OUTPUT_SYMLINK_LIMIT,
    )
    .map_err(export_error)
}

fn export_error(error: NonAliasingWriteError) -> CacheFileError {
    match error {
        NonAliasingWriteError::Alias { output, protected } => {
            CacheFileError::OutputAliasesManagedPath {
                output_path: output,
                managed_path: protected,
            }
        }
        NonAliasingWriteError::InvalidTarget { path, source } => {
            CacheFileError::WriteTemp { path, source }
        }
        NonAliasingWriteError::CreateDirectory { path, source } => {
            CacheFileError::CreateDirectory { path, source }
        }
        NonAliasingWriteError::Write { path, source } => {
            CacheFileError::WriteOutput { path, source }
        }
        NonAliasingWriteError::Sync { path, source } => CacheFileError::SyncOutput { path, source },
    }
}
