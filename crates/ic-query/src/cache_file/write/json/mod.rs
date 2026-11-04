//! Module: cache_file::write::json
//!
//! Responsibility: stream JSON through the confined atomic managed-file writer.
//! Does not own: report schemas, cache policy, or owner-specific errors.
//! Boundary: validates serialization before mutating the filesystem and preserves atomic replace.

use crate::cache_file::{CacheFileError, write_managed_file_atomically};
use ic_host_artifacts::artifact::BoundedWriter;
#[cfg(feature = "subnet-catalog-host")]
use ic_host_artifacts::artifact::{ArtifactIdentity, HashingWriter};
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
};

/// Return the canonical compact JSON byte length without retaining encoded bytes.
#[cfg(any(feature = "nns-host", feature = "subnet-catalog-host", test))]
pub fn canonical_json_serialized_len<T>(value: &T) -> Result<u64, serde_json::Error>
where
    T: Serialize + ?Sized,
{
    let mut writer = BoundedWriter::new(io::sink(), u64::MAX);
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.bytes_written())
}

/// Hash the canonical compact JSON encoding without retaining encoded bytes.
#[cfg(feature = "subnet-catalog-host")]
pub fn canonical_json_sha256<T>(value: &T) -> Result<[u8; 32], serde_json::Error>
where
    T: Serialize + ?Sized,
{
    Ok(*canonical_json_identity(value)?.sha256.as_bytes())
}

/// Return the byte count and SHA-256 of canonical compact JSON without retaining encoded bytes.
#[cfg(feature = "subnet-catalog-host")]
pub fn canonical_json_identity<T>(value: &T) -> Result<ArtifactIdentity, serde_json::Error>
where
    T: Serialize + ?Sized,
{
    let mut writer = HashingWriter::new(io::sink(), u64::MAX);
    serde_json::to_writer(&mut writer, value)?;
    Ok(writer.into_parts().1)
}

/// Serialize pretty JSON without retaining a complete encoded copy and publish it atomically.
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
    let mut counter = BoundedWriter::new(io::sink(), maximum_bytes);
    if let Err(source) = serde_json::to_writer_pretty(&mut counter, value) {
        if counter.limit_exceeded() {
            return Err(write_error(CacheFileError::WriteLimitExceeded {
                path: path.to_path_buf(),
                maximum: maximum_bytes,
            }));
        }
        return Err(serialize_error(path.to_path_buf(), source));
    }
    write_managed_file_atomically(cache_root, path, |file| {
        serde_json::to_writer_pretty(BoundedWriter::new(file, maximum_bytes), value)
    })
    .map_err(write_error)
}

/// Publish a complete collection using its shared byte ceiling and owner-labelled errors.
#[cfg(any(feature = "icrc-host", feature = "nns-host", feature = "sns-host"))]
pub fn write_collection_snapshot<T: Serialize>(
    component: &'static str,
    cache_root: &Path,
    path: &Path,
    value: &T,
) -> Result<(), crate::cache_file::HostCacheError> {
    write_managed_json_pretty_atomically(
        cache_root,
        path,
        value,
        crate::cache_file::MAX_COLLECTION_SNAPSHOT_BYTES,
        |path, source| crate::cache_file::HostCacheError::serialize_cache(component, path, source),
        |source| crate::cache_file::HostCacheError::operation(component, source),
    )
}

#[cfg(test)]
mod tests;
