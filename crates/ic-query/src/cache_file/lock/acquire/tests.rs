use super::*;

pub fn inspect_refresh_lock(cache_root: &Path, path: &Path) -> Result<bool, CacheFileError> {
    let file =
        crate::cache_file::confined::open_managed_file(cache_root, path)?.ok_or_else(|| {
            CacheFileError::ReadRefreshLock {
                path: path.to_path_buf(),
                source: io::Error::new(
                    io::ErrorKind::NotFound,
                    "refresh coordination file disappeared",
                ),
            }
        })?;
    inspect_open_refresh_lock(path, &file)
}
