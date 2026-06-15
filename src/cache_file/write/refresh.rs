use super::{create_parent_directory, write_text_atomically, write_text_output};
use crate::cache_file::{
    CacheFileError,
    lock::{RefreshLockRequest, with_refresh_lock},
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct RefreshCacheWriteRequest<'a, T> {
    pub cache_path: &'a Path,
    pub lock_path: &'a Path,
    pub network: &'a str,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<&'a Path>,
    pub report: &'a T,
}

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
    create_parent_directory(request.cache_path).map_err(&cache_error)?;
    with_refresh_lock(
        RefreshLockRequest {
            lock_path: request.lock_path,
            target_path: request.cache_path,
            network: request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        &cache_error,
        || {
            let replaced_existing_cache = request.cache_path.is_file();
            let report_json = serde_json::to_string_pretty(request.report)
                .map_err(|source| serialize_cache(request.cache_path.to_path_buf(), source))?;
            if let Some(output_path) = request.output_path {
                write_text_output(output_path, &report_json).map_err(&cache_error)?;
            }
            if !request.dry_run {
                write_text_atomically(request.cache_path, &report_json).map_err(&cache_error)?;
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
