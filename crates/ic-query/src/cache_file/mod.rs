//! Module: cache_file
//!
//! Responsibility: shared cache-file IO, locking, and missing-cache policy.
//! Does not own: command-specific cache schemas, report DTOs, or live refreshes.
//! Boundary: exposes reusable cache mechanics used by NNS and SNS report modules.

mod error;
#[cfg(feature = "host")]
mod json;
mod lock;
mod policy;
#[cfg(all(test, feature = "host"))]
mod tests;
mod write;

pub use error::{CacheFileError, HostCacheError};
#[cfg(feature = "host")]
pub use json::HostJsonCacheErrorMapper;
#[cfg(feature = "host")]
pub use json::{
    CachedJsonReport, JsonCacheReport, LoadJsonCacheErrorMapper, LoadJsonCacheRequest,
    load_json_cache, load_json_cache_strict,
};
pub use lock::{RefreshLockRequest, with_refresh_lock};
pub use policy::load_or_refresh_missing_cache;
#[cfg(feature = "host")]
pub use policy::load_or_refresh_stale_cache;
#[cfg(feature = "host")]
pub use write::{RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache};
pub use write::{create_parent_directory, write_text_atomically, write_text_output};
