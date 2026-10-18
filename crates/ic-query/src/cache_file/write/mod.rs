//! Module: cache_file::write
//!
//! Responsibility: shared atomic and refresh-cache write helpers.
//! Does not own: JSON cache schemas, refresh locking internals, or report construction.
//! Boundary: exposes parent creation, atomic text writes, and refresh publication helpers.

#[cfg(feature = "subnet-catalog-host")]
use crate::cache_file::CacheFileError;
#[cfg(feature = "subnet-catalog-host")]
use std::path::Path;

mod json;
#[cfg(feature = "subnet-catalog-host")]
mod output;
#[cfg(feature = "nns-host")]
mod refresh;
#[cfg(all(test, feature = "subnet-catalog-host"))]
mod tests;

#[cfg(feature = "certified-subnet-catalog-host")]
pub use json::canonical_json_matches;
#[cfg(feature = "subnet-catalog-host")]
pub use json::canonical_json_serialized_len;
#[cfg(feature = "subnet-catalog-host")]
pub use json::canonical_json_sha256;
pub use json::write_managed_json_pretty_atomically;
#[cfg(feature = "subnet-catalog-host")]
pub use output::{validate_output_path, write_text_output};
#[cfg(feature = "nns-host")]
pub use refresh::{RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache};

/// Reject an encoded managed file that exceeds its owner's read ceiling.
#[cfg(feature = "subnet-catalog-host")]
pub fn ensure_managed_write_size(
    path: &Path,
    length: u64,
    maximum: u64,
) -> Result<(), CacheFileError> {
    if length > maximum {
        return Err(CacheFileError::WriteLimitExceeded {
            path: path.to_path_buf(),
            maximum,
        });
    }
    Ok(())
}
