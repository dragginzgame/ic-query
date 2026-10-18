//! Descriptor ownership for one guarded refresh; coordination sidecars remain in place.

use crate::cache_file::CacheFileError;
use std::{fs::File, path::PathBuf};

///
/// RefreshLockGuard
///
/// Exclusive kernel lock explicitly released on completion or guard drop.
///

#[derive(Debug)]
pub(super) struct RefreshLockGuard {
    file: File,
    path: PathBuf,
    released: bool,
}

impl RefreshLockGuard {
    pub(super) const fn new(file: File, path: PathBuf) -> Self {
        Self {
            file,
            path,
            released: false,
        }
    }

    pub(super) fn finish<T, E>(
        mut self,
        result: Result<T, E>,
        cache_error: impl FnOnce(CacheFileError) -> E,
    ) -> Result<T, E> {
        if result.is_ok() {
            ic_host_fs::durable::unlock(&self.file).map_err(|source| {
                cache_error(CacheFileError::LockRefreshLock {
                    path: self.path.clone(),
                    source,
                })
            })?;
            self.released = true;
        }
        result
    }
}

impl Drop for RefreshLockGuard {
    fn drop(&mut self) {
        if !self.released {
            // A duplicated or inherited descriptor must not prolong this refresh.
            // Preserve action failures; final close remains the unlock-error fallback.
            let _ = ic_host_fs::durable::unlock(&self.file);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RefreshLockGuard;
    use crate::test_support::temp_dir;
    use std::fs::{self, File};

    #[test]
    fn drop_releases_lock_even_with_a_duplicate_descriptor() {
        let root = temp_dir("ic-query-drop-duplicated-refresh-lock");
        let path = root.join("refresh.lock");
        let file = File::create(&path).unwrap();
        ic_host_fs::durable::try_lock_exclusive(&file).unwrap();
        let duplicate = file.try_clone().unwrap();
        let guard = RefreshLockGuard::new(file, path.clone());
        let observer = File::open(&path).unwrap();
        assert_eq!(
            ic_host_fs::durable::try_lock_exclusive(&observer)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::WouldBlock
        );

        drop(guard);

        ic_host_fs::durable::try_lock_exclusive(&observer).unwrap();
        ic_host_fs::durable::unlock(&observer).unwrap();
        drop(duplicate);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn finish_releases_duplicates_and_preserves_action_results() {
        let root = temp_dir("ic-query-finish-duplicated-refresh-lock");
        let path = root.join("refresh.lock");
        for outcome in [Ok(7), Err("collection failed")] {
            let file = File::create(&path).unwrap();
            ic_host_fs::durable::try_lock_exclusive(&file).unwrap();
            let duplicate = file.try_clone().unwrap();
            let guard = RefreshLockGuard::new(file, path.clone());

            assert_eq!(guard.finish(outcome, |_| "lock release failed"), outcome);

            let observer = File::open(&path).unwrap();
            ic_host_fs::durable::try_lock_exclusive(&observer).unwrap();
            ic_host_fs::durable::unlock(&observer).unwrap();
            drop(duplicate);
        }
        fs::remove_dir_all(root).unwrap();
    }
}
