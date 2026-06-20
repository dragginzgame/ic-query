//! Module: cache_file::write::atomic
//!
//! Responsibility: atomically replace text cache files.
//! Does not own: refresh locking, JSON serialization, or output-path writes.
//! Boundary: writes a temp file, syncs it, renames it, and syncs the directory.

use super::path::{sync_directory, target_directory};
use crate::cache_file::CacheFileError;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static ATOMIC_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

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
