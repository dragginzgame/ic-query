//! Module: cache_file::write
//!
//! Responsibility: shared atomic and refresh-cache write helpers.
//! Does not own: JSON cache schemas, refresh locking internals, or report construction.
//! Boundary: exposes parent creation, atomic text writes, and refresh publication helpers.

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
mod json;
#[cfg(any(feature = "nns-host", feature = "subnet-catalog-host"))]
mod output;
#[cfg(any(feature = "nns-host", feature = "subnet-catalog-host"))]
mod path;
#[cfg(feature = "nns-host")]
mod refresh;

#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "nns-topology-host",
    feature = "sns-host"
))]
pub use json::write_managed_json_pretty_atomically;
#[cfg(any(feature = "nns-host", feature = "subnet-catalog-host"))]
pub use output::write_text_output;
#[cfg(feature = "nns-host")]
pub use refresh::{RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache};
