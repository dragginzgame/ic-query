//! Module: cache_file
//!
//! Responsibility: shared cache-file IO, locking, and refresh policy.
//! Does not own: command-specific cache schemas, report DTOs, or live refreshes.
//! Boundary: exposes reusable cache mechanics used by NNS and SNS report modules.

mod confined;
mod error;
#[cfg(feature = "host")]
mod json;
mod lock;
mod policy;
#[cfg(all(test, feature = "host"))]
mod tests;
mod write;

#[cfg(feature = "host")]
pub use confined::{
    collect_managed_collection_files, collect_managed_files, open_managed_file, read_managed_file,
};
pub use confined::{
    create_managed_parent_directory, managed_file_exists, read_managed_text,
    write_managed_text_atomically,
};
pub use error::{CacheFileError, HostCacheError};
#[cfg(feature = "host")]
pub use json::HostJsonCacheErrorMapper;
#[cfg(feature = "host")]
pub use json::{
    CachedJsonReport, JsonCacheReport, LoadJsonCacheErrorMapper, LoadJsonCacheRequest,
    OwnerJsonCacheErrorMapper, load_json_cache, load_json_cache_strict,
};
#[cfg(feature = "host")]
pub use lock::{RefreshLockEvidence, inspect_refresh_lock};
pub use lock::{RefreshLockRequest, with_refresh_lock};
#[cfg(feature = "host")]
pub use policy::{CacheRefreshReason, load_or_refresh_cache_with_error_policy};
#[cfg(feature = "host")]
pub use policy::{
    host_cache_refresh_reason, load_or_refresh_missing_cache,
    load_or_refresh_stale_cache_with_error_policy,
};
pub use write::write_text_output;
#[cfg(feature = "host")]
pub use write::{RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache};
