//! Module: cache_file::write::json
//!
//! Responsibility: stream JSON through the confined atomic managed-file writer.
//! Does not own: report schemas, cache policy, or owner-specific errors.
//! Boundary: validates serialization before mutating the filesystem and preserves atomic replace.

use crate::cache_file::{CacheFileError, write_managed_file_atomically};
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
};

/// Serialize pretty JSON without retaining a complete encoded copy and publish it atomically.
pub fn write_managed_json_pretty_atomically<T, Error>(
    cache_root: &Path,
    path: &Path,
    value: &T,
    serialize_error: impl FnOnce(PathBuf, serde_json::Error) -> Error,
    write_error: impl FnOnce(CacheFileError) -> Error,
) -> Result<(), Error>
where
    T: Serialize,
{
    serde_json::to_writer_pretty(io::sink(), value)
        .map_err(|source| serialize_error(path.to_path_buf(), source))?;
    write_managed_file_atomically(cache_root, path, |file| {
        serde_json::to_writer_pretty(file, value).map_err(io::Error::other)
    })
    .map_err(write_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cache_file::read_managed_text, test_support::temp_dir};
    use serde::{Serializer, ser::Error as _};
    use std::fs;

    struct FailingSerialization;

    impl Serialize for FailingSerialization {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("fixture serialization failure"))
        }
    }

    #[test]
    fn streamed_json_write_preserves_bytes_and_existing_file_on_serialization_failure() {
        let root = temp_dir("ic-query-streamed-json-write");
        let path = root.join("sns/ic/root/neurons/full.json");
        let value = serde_json::json!({"schema_version": 1, "rows": ["a", "b"]});

        write_managed_json_pretty_atomically(
            &root,
            &path,
            &value,
            |_, source| source.to_string(),
            |source| source.to_string(),
        )
        .expect("write streamed JSON");
        assert_eq!(
            read_managed_text(&root, &path).expect("read streamed JSON"),
            Some(serde_json::to_string_pretty(&value).expect("encode expected JSON"))
        );

        let error = write_managed_json_pretty_atomically(
            &root,
            &path,
            &FailingSerialization,
            |_, source| source.to_string(),
            |source| source.to_string(),
        )
        .expect_err("serialization failure is returned");
        assert!(error.contains("fixture serialization failure"));
        assert_eq!(
            read_managed_text(&root, &path).expect("read preserved JSON"),
            Some(serde_json::to_string_pretty(&value).expect("encode expected JSON"))
        );

        let _ = fs::remove_dir_all(root);
    }
}
