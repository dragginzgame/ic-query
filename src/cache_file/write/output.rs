use super::path::create_parent_directory;
use crate::cache_file::CacheFileError;
use std::{fs, io::Write, path::Path};

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
