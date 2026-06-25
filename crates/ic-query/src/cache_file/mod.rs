//! Module: cache_file
//!
//! Responsibility: shared cache-file IO, locking, and missing-cache policy.
//! Does not own: command-specific cache schemas, report DTOs, or live refreshes.
//! Boundary: exposes reusable cache mechanics used by NNS and SNS report modules.

mod error;
mod json;
mod lock;
mod policy;
#[cfg(test)]
mod tests;
mod write;

pub use error::CacheFileError;
pub use json::{
    CachedJsonReport, JsonCacheReport, LoadJsonCacheErrorMapper, LoadJsonCacheRequest,
    announce_cache_refresh, load_json_cache,
};
pub use lock::{RefreshLockRequest, with_refresh_lock};
pub use policy::load_or_refresh_missing_cache;
pub use write::{
    RefreshCacheWriteRequest, RefreshCacheWriteResult, create_parent_directory,
    write_json_refresh_cache, write_text_atomically, write_text_output,
};
