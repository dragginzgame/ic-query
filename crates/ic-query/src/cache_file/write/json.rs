//! Module: cache_file::write::json
//!
//! Responsibility: stream JSON through the confined atomic managed-file writer.
//! Does not own: report schemas, cache policy, or owner-specific errors.
//! Boundary: validates serialization before mutating the filesystem and preserves atomic replace.

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host",
    test
))]
use crate::cache_file::{CacheFileError, write_managed_file_atomically};
use serde::Serialize;
use std::io;

#[cfg(any(feature = "certified-subnet-catalog-host", test))]
use std::io::Write;

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host",
    test
))]
use std::path::{Path, PathBuf};

/// Return the canonical compact JSON byte length without retaining encoded bytes.
#[cfg(any(feature = "certified-subnet-catalog-host", test))]
pub fn canonical_json_serialized_len<T>(value: &T) -> Result<u64, serde_json::Error>
where
    T: Serialize + ?Sized,
{
    let mut writer = CountingWriter::default();
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.bytes)
}

/// Return whether `bytes` are the exact canonical compact JSON encoding of `value`.
#[cfg(any(feature = "certified-subnet-catalog-host", test))]
pub fn canonical_json_matches<T>(value: &T, bytes: &[u8]) -> Result<bool, serde_json::Error>
where
    T: Serialize + ?Sized,
{
    let mut writer = MatchingWriter::new(bytes);
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.is_complete_match())
}

/// Preserve an underlying writer error kind when adapting JSON serialization to atomic IO.
pub fn json_error_to_io(error: serde_json::Error) -> io::Error {
    match error.io_error_kind() {
        Some(kind) => io::Error::new(kind, error),
        None => io::Error::other(error),
    }
}

/// Serialize pretty JSON without retaining a complete encoded copy and publish it atomically.
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host",
    test
))]
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
        serde_json::to_writer_pretty(file, value).map_err(json_error_to_io)
    })
    .map_err(write_error)
}

#[derive(Default)]
#[cfg(any(feature = "certified-subnet-catalog-host", test))]
struct CountingWriter {
    bytes: u64,
}

#[cfg(any(feature = "certified-subnet-catalog-host", test))]
impl Write for CountingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length = u64::try_from(buffer.len())
            .map_err(|_| io::Error::other("canonical JSON byte count exceeds u64"))?;
        self.bytes = self
            .bytes
            .checked_add(length)
            .ok_or_else(|| io::Error::other("canonical JSON byte count exceeds u64"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(any(feature = "certified-subnet-catalog-host", test))]
struct MatchingWriter<'a> {
    expected: &'a [u8],
    position: usize,
    matches: bool,
}

#[cfg(any(feature = "certified-subnet-catalog-host", test))]
impl<'a> MatchingWriter<'a> {
    const fn new(expected: &'a [u8]) -> Self {
        Self {
            expected,
            position: 0,
            matches: true,
        }
    }

    const fn is_complete_match(&self) -> bool {
        self.matches && self.position == self.expected.len()
    }
}

#[cfg(any(feature = "certified-subnet-catalog-host", test))]
impl Write for MatchingWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let end = self
            .position
            .checked_add(buffer.len())
            .ok_or_else(|| io::Error::other("canonical JSON byte position exceeds usize"))?;
        if self.expected.get(self.position..end) != Some(buffer) {
            self.matches = false;
        }
        self.position = end;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
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

    #[test]
    fn canonical_json_helpers_count_and_match_without_encoding_a_second_copy() {
        let value = serde_json::json!({"schema_version": 1, "rows": ["a", "b"]});
        let canonical = serde_json::to_vec(&value).expect("encode canonical fixture");

        assert_eq!(
            canonical_json_serialized_len(&value).expect("count canonical JSON"),
            u64::try_from(canonical.len()).expect("fixture length fits u64")
        );
        assert!(canonical_json_matches(&value, &canonical).expect("match canonical JSON"));
        assert!(!canonical_json_matches(&value, b"{}").expect("reject different JSON"));
    }
}
