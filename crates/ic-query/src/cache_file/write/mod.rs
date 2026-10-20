//! Module: cache_file::write
//!
//! Responsibility: shared atomic and refresh-cache write helpers.
//! Does not own: JSON cache schemas, refresh locking internals, or report construction.
//! Boundary: exposes parent creation, atomic text writes, and refresh publication helpers.

mod json;
#[cfg(feature = "subnet-catalog-host")]
mod output;
#[cfg(feature = "subnet-catalog-host")]
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
#[cfg(feature = "subnet-catalog-host")]
pub use refresh::{RefreshCacheWriteRequest, write_json_refresh_outputs};
#[cfg(feature = "nns-host")]
pub use refresh::{RefreshCacheWriteResult, write_json_refresh_cache};
