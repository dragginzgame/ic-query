//! Module: cache_file
//!
//! Responsibility: shared cache-file IO, locking, and refresh policy.
//! Does not own: command-specific cache schemas, report DTOs, or live refreshes.
//! Boundary: exposes reusable cache mechanics used by NNS and SNS report modules.

mod confined;
mod error;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
mod json;
mod lock;
mod policy;
#[cfg(all(test, feature = "host"))]
mod tests;
mod write;

#[cfg(feature = "sns-host")]
pub use confined::collect_managed_collection_files;
#[cfg(any(feature = "icrc-host", feature = "nns-host", feature = "sns-host"))]
pub use confined::read_managed_file;
#[cfg(feature = "host")]
pub use confined::{collect_managed_files, open_managed_file};
pub use confined::{
    create_managed_parent_directory, managed_file_exists, read_managed_text,
    write_managed_text_atomically,
};
pub use error::{CacheFileError, HostCacheError};
#[cfg(any(feature = "icrc-host", feature = "nns-topology-host"))]
pub use json::HostJsonCacheErrorMapper;
#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
pub use json::OwnerJsonCacheErrorMapper;
#[cfg(feature = "nns-topology-host")]
pub use json::load_json_cache;
#[cfg(all(
    feature = "icrc-host",
    not(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))
))]
pub use json::load_json_cache_strict;
#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
pub use json::{CachedJsonReport, LoadJsonCacheErrorMapper, load_json_cache_strict};
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use json::{JsonCacheReport, LoadJsonCacheRequest};
pub use lock::RefreshLockRequest;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use lock::with_refresh_lock;
#[cfg(feature = "subnet-catalog-host")]
pub use lock::with_refresh_lock_async;
#[cfg(feature = "host")]
pub use lock::{RefreshLockEvidence, inspect_refresh_lock};
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use policy::CacheRefreshReason;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host"
))]
pub use policy::load_or_refresh_cache_with_error_policy;
#[cfg(feature = "sns-host")]
pub use policy::load_or_refresh_missing_cache;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use policy::{host_cache_refresh_reason, load_or_refresh_stale_cache_with_error_policy};
#[cfg(any(feature = "nns-host", feature = "subnet-catalog-host"))]
pub use write::write_text_output;
#[cfg(feature = "nns-host")]
pub use write::{RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache};
