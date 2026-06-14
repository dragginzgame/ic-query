use super::{
    CacheFileError,
    lock::{RefreshLockRequest, with_refresh_lock},
};
use serde::Serialize;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static ATOMIC_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

///
/// RefreshCacheWriteRequest
///
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

///
/// RefreshCacheWriteResult
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshCacheWriteResult {
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
}

fn create_directory(path: &Path) -> Result<(), CacheFileError> {
    fs::create_dir_all(path).map_err(|source| CacheFileError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

pub fn create_parent_directory(target_path: &Path) -> Result<(), CacheFileError> {
    create_directory(target_directory(target_path)?)
}

pub fn write_text_atomically(target_path: &Path, contents: &str) -> Result<(), CacheFileError> {
    let target_dir = target_directory(target_path)?;
    let target_file = target_path
        .file_name()
        .and_then(|file| file.to_str())
        .unwrap_or("cache");
    let temp_path = atomic_temp_path(target_dir, target_file);
    let write_result = (|| {
        let mut temp = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|source| CacheFileError::WriteTemp {
                path: temp_path.clone(),
                source,
            })?;
        temp.write_all(contents.as_bytes())
            .map_err(|source| CacheFileError::WriteTemp {
                path: temp_path.clone(),
                source,
            })?;
        temp.sync_all().map_err(|source| CacheFileError::SyncTemp {
            path: temp_path.clone(),
            source,
        })?;
        Ok(())
    })();
    if let Err(err) = write_result {
        remove_temp_file(&temp_path);
        return Err(err);
    }
    if let Err(source) = fs::rename(&temp_path, target_path) {
        remove_temp_file(&temp_path);
        return Err(CacheFileError::Replace {
            temp_path,
            target_path: target_path.to_path_buf(),
            source,
        });
    }
    sync_directory(target_dir)
}

pub fn write_text_output(output_path: &Path, contents: &str) -> Result<(), CacheFileError> {
    create_parent_directory(output_path)?;
    let mut output =
        fs::File::create(output_path).map_err(|source| CacheFileError::WriteOutput {
            path: output_path.to_path_buf(),
            source,
        })?;
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

fn sync_directory(path: &Path) -> Result<(), CacheFileError> {
    fs::File::open(path)
        .and_then(|dir| dir.sync_all())
        .map_err(|source| CacheFileError::SyncDirectory {
            path: path.to_path_buf(),
            source,
        })
}

fn target_directory(target_path: &Path) -> Result<&Path, CacheFileError> {
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

#[must_use]
fn atomic_temp_path(target_dir: &Path, target_file: &str) -> PathBuf {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = ATOMIC_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    target_dir.join(format!(
        "{target_file}.tmp.{}.{}.{}",
        std::process::id(),
        now_nanos,
        counter
    ))
}

fn remove_temp_file(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn atomic_write_rejects_parentless_target_path() {
        let err = write_text_atomically(Path::new("/"), "data").expect_err("invalid path");

        match err {
            CacheFileError::WriteTemp { path, source } => {
                assert_eq!(path, PathBuf::from("/"));
                assert_eq!(source.kind(), ErrorKind::InvalidInput);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn relative_single_file_target_uses_current_directory() {
        assert_eq!(
            target_directory(Path::new("cache.json")).unwrap(),
            Path::new(".")
        );
    }
}
