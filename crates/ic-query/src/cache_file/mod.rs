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

/// Maximum encoded bytes admitted for inventories and bounded report snapshots.
#[cfg(any(
    feature = "dashboard-host",
    feature = "subnet-catalog-host",
    feature = "sns-host"
))]
pub const MAX_JSON_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;

/// Maximum encoded bytes admitted for complete Governance and account histories.
#[cfg(any(feature = "icrc-host", feature = "nns-host", feature = "sns-host"))]
pub const MAX_COLLECTION_SNAPSHOT_BYTES: u64 = 512 * 1024 * 1024;

/// Maximum encoded bytes admitted for refresh-attempt sidecars.
#[cfg(any(feature = "icrc-host", feature = "nns-host", feature = "sns-host"))]
pub const MAX_REFRESH_ATTEMPT_BYTES: u64 = 1024 * 1024;

pub use confined::BoundedManagedFileReadError;
pub use confined::read_bounded_managed_file;
pub use confined::write_managed_file_atomically;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use confined::{ManagedReadBudget, read_opened_file_bounded};
#[cfg(feature = "subnet-catalog-host")]
pub use write::canonical_json_identity;
#[cfg(feature = "certified-subnet-catalog-host")]
pub use write::canonical_json_matches;
#[cfg(feature = "subnet-catalog-host")]
pub use write::canonical_json_serialized_len;
#[cfg(feature = "subnet-catalog-host")]
pub use write::canonical_json_sha256;
pub use write::write_managed_json_pretty_atomically;

#[cfg(feature = "sns-host")]
pub use confined::collect_managed_collection_files;
#[cfg(any(
    feature = "host",
    feature = "icrc-host",
    feature = "nns-host",
    feature = "sns-host"
))]
pub use confined::open_managed_file;
#[cfg(any(feature = "subnet-catalog-host", test))]
pub use confined::write_managed_text_atomically;
#[cfg(feature = "certified-subnet-catalog-host")]
pub use confined::{
    ManagedDirectoryFile, remove_managed_regular_file, scan_managed_directory_files,
};
#[cfg(feature = "host")]
pub use confined::{collect_managed_files, read_bounded_stream};
pub use confined::{create_managed_parent_directory, managed_file_exists};
pub use error::{CacheFileError, HostCacheError};
#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
pub use json::CachedJsonReport;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use json::{JsonCacheReport, LoadJsonCacheRequest, load_json_cache};
pub use lock::RefreshLockRequest;
#[cfg(feature = "host")]
pub use lock::inspect_open_refresh_lock;
#[cfg(test)]
pub use lock::inspect_refresh_lock;
#[cfg(feature = "subnet-catalog-host")]
pub use lock::with_refresh_lock;
pub use lock::with_refresh_lock_async;
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
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use policy::{host_cache_refresh_reason, load_or_refresh_cache};
#[cfg(feature = "subnet-catalog-host")]
pub use write::validate_output_path;
#[cfg(feature = "subnet-catalog-host")]
pub use write::{RefreshCacheWriteRequest, write_json_refresh_outputs};
#[cfg(feature = "nns-host")]
pub use write::{RefreshCacheWriteResult, write_json_refresh_cache};
