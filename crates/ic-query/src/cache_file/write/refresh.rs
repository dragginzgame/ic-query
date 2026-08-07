//! Module: cache_file::write::refresh
//!
//! Responsibility: publish refresh reports to cache and optional output files.
//! Does not own: command-specific reports, cache paths, or live refreshes.
//! Boundary: serializes refresh output under the shared refresh-lock guard.

use super::write_text_output;
use crate::cache_file::{
    CacheFileError, create_managed_parent_directory,
    lock::{RefreshLockRequest, with_refresh_lock},
    managed_file_exists, write_managed_json_pretty_atomically, write_managed_text_atomically,
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
    pub network: &'a str,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<&'a Path>,
    pub report: &'a T,
}

///
/// RefreshCacheWriteResult
///
/// File paths and write status returned after publishing a refresh result.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshCacheWriteResult {
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
}

pub fn write_json_refresh_cache<T, E>(
    request: RefreshCacheWriteRequest<'_, T>,
    cache_error: impl Fn(CacheFileError) -> E,
    serialize_cache: impl Fn(PathBuf, serde_json::Error) -> E,
) -> Result<RefreshCacheWriteResult, E>
where
    T: Serialize,
{
    create_managed_parent_directory(request.cache_root, request.cache_path)
        .map_err(&cache_error)?;
    with_refresh_lock(
        RefreshLockRequest {
            cache_root: request.cache_root,
            lock_path: request.lock_path,
            target_path: request.cache_path,
            network: request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        &cache_error,
        || {
            let replaced_existing_cache =
                managed_file_exists(request.cache_root, request.cache_path)
                    .map_err(&cache_error)?;
            if let Some(output_path) = request.output_path {
                let report_json = serde_json::to_string_pretty(request.report)
                    .map_err(|source| serialize_cache(request.cache_path.to_path_buf(), source))?;
                write_text_output(output_path, &report_json).map_err(&cache_error)?;
                if !request.dry_run {
                    write_managed_text_atomically(
                        request.cache_root,
                        request.cache_path,
                        &report_json,
                    )
                    .map_err(&cache_error)?;
                }
            } else if request.dry_run {
                serde_json::to_writer_pretty(io::sink(), request.report)
                    .map_err(|source| serialize_cache(request.cache_path.to_path_buf(), source))?;
            } else {
                write_managed_json_pretty_atomically(
                    request.cache_root,
                    request.cache_path,
                    request.report,
                    &serialize_cache,
                    &cache_error,
                )?;
            }
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
