use std::io::Write;

use super::*;
use crate::{cache_file::confined::read_bounded_managed_file, tests::temp_dir};
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
    write_managed_file_atomically(&root, &path, |file| file.write_all(b"original"))
        .expect("seed managed cache");
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
            source: CacheFileError::PublishManagedFile {
                published: false,
                source,
                cleanup_error: None,
                ..
            },
            ..
        } if matches!(
            source.get_ref().and_then(|error| error.downcast_ref::<ic_host_artifacts::artifact::WriterError>()),
            Some(ic_host_artifacts::artifact::WriterError::LimitExceeded { limit: 16 })
        )
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
fn canonical_json_helpers_count_and_hash_without_encoding_a_second_copy() {
    let value = serde_json::json!({"schema_version": 1, "rows": ["escape\nλ", "b"]});
    let canonical = serde_json::to_vec(&value).expect("encode canonical fixture");

    #[cfg(feature = "subnet-catalog-host")]
    {
        let identity = canonical_json_identity(&value).expect("canonical identity");
        assert_eq!(identity.bytes, canonical.len() as u64);
        assert_eq!(
            identity.sha256,
            ic_host_artifacts::artifact::Sha256Digest::compute(&canonical)
        );
        assert!(canonical_json_identity(&FailingSerialization).is_err());
    }
    assert_eq!(
        canonical_json_serialized_len(&value).expect("count canonical JSON"),
        u64::try_from(canonical.len()).expect("fixture length fits u64")
    );
}
