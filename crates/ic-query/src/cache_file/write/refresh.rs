//! Module: cache_file::write::refresh
//!
//! Responsibility: publish refresh reports to cache and optional output files.
//! Does not own: command-specific reports, cache paths, or live refreshes.
//! Boundary: callers acquire a refresh lock before selecting preview, export and publication.

#[cfg(feature = "nns-host")]
use super::validate_output_path;
use super::write_text_output;
use crate::cache_file::{
    CacheFileError, write_managed_json_pretty_atomically, write_managed_text_atomically,
};
#[cfg(feature = "nns-host")]
use crate::cache_file::{
    RefreshLockRequest, create_managed_parent_directory, managed_file_exists, with_refresh_lock,
};
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
};

///
/// RefreshCacheWriteRequest
///
/// Inputs for publishing a refresh result into the shared JSON cache.
///

#[derive(Clone, Copy, Debug)]
pub struct RefreshCacheWriteRequest<'a, T> {
    /// Capability root that confines the managed cache and lock paths.
    pub cache_root: &'a Path,
    pub cache_path: &'a Path,
    pub lock_path: &'a Path,

    pub dry_run: bool,
    /// Optional export that must not alias the managed cache or refresh lock.
    pub output_path: Option<&'a Path>,
    pub report: &'a T,
}

///
/// RefreshCacheWriteResult
///
/// File paths and write status returned after publishing a refresh result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(feature = "nns-host")]
pub struct RefreshCacheWriteResult {
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
}

#[cfg(feature = "nns-host")]
pub fn write_json_refresh_cache<T, E>(
    request: RefreshCacheWriteRequest<'_, T>,
    cache_error: impl Fn(CacheFileError) -> E,
    serialize_cache: impl Fn(PathBuf, serde_json::Error) -> E,
) -> Result<RefreshCacheWriteResult, E>
where
    T: Serialize,
{
    let managed_paths = [request.cache_path, request.lock_path];
    if let Some(output_path) = request.output_path {
        validate_output_path(output_path, &managed_paths).map_err(&cache_error)?;
    }
    create_managed_parent_directory(request.cache_root, request.cache_path)
        .map_err(&cache_error)?;
    with_refresh_lock(
        RefreshLockRequest {
            cache_root: request.cache_root,
            lock_path: request.lock_path,
        },
        &cache_error,
        || {
            let replaced_existing_cache =
                managed_file_exists(request.cache_root, request.cache_path)
                    .map_err(&cache_error)?;
            write_json_refresh_outputs(
                &request,
                &[],
                |_, error| cache_error(error),
                &serialize_cache,
            )?;
            Ok(RefreshCacheWriteResult {
                cache_path: request.cache_path.display().to_string(),
                refresh_lock_path: request.lock_path.display().to_string(),
                output_path: request.output_path.map(|path| path.display().to_string()),
                replaced_existing_cache,
                wrote_cache: !request.dry_run,
            })
        },
    )
}

/// Serialize, optionally export, and publish a report while the caller holds its lock.
/// Additional protected paths supplement the cache and refresh-lock paths.
pub fn write_json_refresh_outputs<T, E>(
    request: &RefreshCacheWriteRequest<'_, T>,
    additional_managed_paths: &[&Path],
    cache_error: impl Fn(&Path, CacheFileError) -> E,
    serialize_cache: impl Fn(PathBuf, serde_json::Error) -> E,
) -> Result<(), E>
where
    T: Serialize,
{
    if let Some(output_path) = request.output_path {
        let report_json = serde_json::to_string_pretty(request.report)
            .map_err(|source| serialize_cache(request.cache_path.to_path_buf(), source))?;
        if !request.dry_run && report_json.len() as u64 > crate::cache_file::MAX_JSON_SNAPSHOT_BYTES
        {
            return Err(cache_error(
                request.cache_path,
                CacheFileError::WriteLimitExceeded {
                    path: request.cache_path.to_path_buf(),
                    maximum: crate::cache_file::MAX_JSON_SNAPSHOT_BYTES,
                },
            ));
        }
        let mut managed_paths = vec![request.cache_path, request.lock_path];
        managed_paths.extend_from_slice(additional_managed_paths);
        write_text_output(output_path, &report_json, &managed_paths)
            .map_err(|error| cache_error(output_path, error))?;
        if !request.dry_run {
            write_managed_text_atomically(request.cache_root, request.cache_path, &report_json)
                .map_err(|error| cache_error(request.cache_path, error))?;
        }
    } else if request.dry_run {
        serde_json::to_writer_pretty(io::sink(), request.report)
            .map_err(|source| serialize_cache(request.cache_path.to_path_buf(), source))?;
    } else {
        write_managed_json_pretty_atomically(
            request.cache_root,
            request.cache_path,
            request.report,
            crate::cache_file::MAX_JSON_SNAPSHOT_BYTES,
            serialize_cache,
            |error| cache_error(request.cache_path, error),
        )?;
    }
    Ok(())
}
