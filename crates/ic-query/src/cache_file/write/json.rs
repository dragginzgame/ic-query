//! Module: cache_file::write::json
//!
//! Responsibility: stream JSON through the confined atomic managed-file writer.
//! Does not own: report schemas, cache policy, or owner-specific errors.
//! Boundary: validates serialization before mutating the filesystem and preserves atomic replace.

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host",
    feature = "sns-host",
    test
))]
use crate::cache_file::{CacheFileError, write_managed_file_atomically};
use serde::Serialize;
#[cfg(feature = "subnet-catalog-host")]
use sha2::{Digest, Sha256};
use std::io;

#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "subnet-catalog-host",
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "sns-host",
    test
))]
use std::io::Write;

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host",
    feature = "sns-host",
    test
))]
use std::path::{Path, PathBuf};

/// Return the canonical compact JSON byte length without retaining encoded bytes.
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "subnet-catalog-host",
    test
))]
pub fn canonical_json_serialized_len<T>(value: &T) -> Result<u64, serde_json::Error>
where
    T: Serialize + ?Sized,
{
    let mut writer = CountingWriter::default();
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.bytes)
}

/// Hash the canonical compact JSON encoding without retaining encoded bytes.
#[cfg(feature = "subnet-catalog-host")]
pub fn canonical_json_sha256<T>(value: &T) -> Result<[u8; 32], serde_json::Error>
where
    T: Serialize + ?Sized,
{
    let mut writer = JsonDigestWriter(Sha256::new());
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.0.finalize().into())
}

#[cfg(feature = "subnet-catalog-host")]
struct JsonDigestWriter(Sha256);

#[cfg(feature = "subnet-catalog-host")]
impl Write for JsonDigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
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
    feature = "subnet-catalog-host",
    feature = "sns-host",
    test
))]
pub fn write_managed_json_pretty_atomically<T, Error>(
    cache_root: &Path,
    path: &Path,
    value: &T,
    maximum_bytes: u64,
    serialize_error: impl FnOnce(PathBuf, serde_json::Error) -> Error,
    write_error: impl FnOnce(CacheFileError) -> Error,
) -> Result<(), Error>
where
    T: Serialize,
{
    let mut counter = LimitedJsonWriter::new(io::sink(), maximum_bytes);
    if let Err(source) = serde_json::to_writer_pretty(&mut counter, value) {
        if counter.exceeded {
            return Err(write_error(CacheFileError::WriteLimitExceeded {
                path: path.to_path_buf(),
                maximum: maximum_bytes,
            }));
        }
        return Err(serialize_error(path.to_path_buf(), source));
    }
    write_managed_file_atomically(cache_root, path, |file| {
        serde_json::to_writer_pretty(LimitedJsonWriter::new(file, maximum_bytes), value)
            .map_err(json_error_to_io)
    })
    .map_err(write_error)
}

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host",
    feature = "sns-host",
    test
))]
struct LimitedJsonWriter<W> {
    writer: W,
    maximum: u64,
    bytes: u64,
    exceeded: bool,
}

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host",
    feature = "sns-host",
    test
))]
impl<W> LimitedJsonWriter<W> {
    const fn new(writer: W, maximum: u64) -> Self {
        Self {
            writer,
            maximum,
            bytes: 0,
            exceeded: false,
        }
    }
}

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "subnet-catalog-host",
    feature = "sns-host",
    test
))]
impl<W: Write> Write for LimitedJsonWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length = u64::try_from(buffer.len()).map_err(io::Error::other)?;
        if self
            .bytes
            .checked_add(length)
            .is_none_or(|bytes| bytes > self.maximum)
        {
            self.exceeded = true;
            return Err(io::Error::other(
                "serialized JSON exceeds the cache write byte limit",
            ));
        }
        let written = self.writer.write(buffer)?;
        self.bytes += u64::try_from(written).map_err(io::Error::other)?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[derive(Default)]
#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "subnet-catalog-host",
    test
))]
struct CountingWriter {
    bytes: u64,
}

#[cfg(any(
    feature = "certified-subnet-catalog-host",
    feature = "subnet-catalog-host",
    test
))]
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
    use crate::{cache_file::confined::read_bounded_managed_file, test_support::temp_dir};
    use serde::{Serializer, ser::Error as _};
    use std::fs;

    #[test]
    fn json_write_limit_preserves_existing_cache_and_accepts_exact_ceiling() {
        let root = temp_dir("ic-query-json-write-limit");
        let path = root.join("full.json");
        let value = serde_json::json!({"rows": ["fixture"]});
        let bytes = serde_json::to_vec_pretty(&value).unwrap();
        let maximum = u64::try_from(bytes.len()).unwrap();
        let write = |path: &Path, maximum| {
            write_managed_json_pretty_atomically(
                &root,
                path,
                &value,
                maximum,
                |path, source| crate::HostCacheError::serialize_cache("fixture", path, source),
                |source| crate::HostCacheError::operation("fixture", source),
            )
        };
        write(&path, maximum).expect("exact ceiling is admitted");
        let error = write(&path, maximum - 1).expect_err("oversized replacement rejected");
        assert!(matches!(error, crate::HostCacheError::Operation {
            source: CacheFileError::WriteLimitExceeded { maximum: actual, .. }, ..
        } if actual == maximum - 1));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let missing = root.join("new/full.json");
        write(&missing, maximum - 1).expect_err("oversized new cache rejected");
        assert!(!missing.parent().unwrap().exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn changed_serialization_cannot_exceed_the_limit_during_atomic_write() {
        struct ChangingSerialization(std::cell::Cell<bool>);

        impl Serialize for ChangingSerialization {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(if self.0.replace(true) {
                    "a larger second serialization that exceeds the write limit"
                } else {
                    "small"
                })
            }
        }

        let root = temp_dir("ic-query-json-second-pass-limit");
        let path = root.join("full.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(&path, b"original").unwrap();
        let error = write_managed_json_pretty_atomically(
            &root,
            &path,
            &ChangingSerialization(std::cell::Cell::new(false)),
            16,
            |path, source| crate::HostCacheError::serialize_cache("fixture", path, source),
            |source| crate::HostCacheError::operation("fixture", source),
        )
        .expect_err("second serialization remains bounded");
        assert!(matches!(
            error,
            crate::HostCacheError::Operation {
                source: CacheFileError::WriteTemp { .. },
                ..
            }
        ));
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

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
            1024,
            |_, source| source.to_string(),
            |source| source.to_string(),
        )
        .expect("write streamed JSON");
        assert_eq!(
            read_bounded_managed_file(&root, &path, 1024, None).expect("read streamed JSON"),
            Some(serde_json::to_vec_pretty(&value).expect("encode expected JSON"))
        );

        let error = write_managed_json_pretty_atomically(
            &root,
            &path,
            &FailingSerialization,
            1024,
            |_, source| source.to_string(),
            |source| source.to_string(),
        )
        .expect_err("serialization failure is returned");
        assert!(error.contains("fixture serialization failure"));
        assert_eq!(
            read_bounded_managed_file(&root, &path, 1024, None).expect("read preserved JSON"),
            Some(serde_json::to_vec_pretty(&value).expect("encode expected JSON"))
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
