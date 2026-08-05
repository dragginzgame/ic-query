//! Module: cache_file::json
//!
//! Responsibility: shared JSON cache loading and validation.
//! Does not own: command-specific cache schemas, refresh execution, or process output.
//! Boundary: exposes schema and network validation helpers for cached reports.

mod errors;
mod load;
mod model;

#[cfg(any(feature = "icrc-host", feature = "nns-topology-host"))]
pub use errors::HostJsonCacheErrorMapper;
#[cfg(any(feature = "dashboard-host", feature = "sns-host"))]
pub use errors::LoadJsonCacheErrorMapper;
#[cfg(any(feature = "dashboard-host", feature = "sns-host"))]
pub use errors::OwnerJsonCacheErrorMapper;
#[cfg(feature = "nns-topology-host")]
pub use load::load_json_cache;
#[cfg(any(
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "sns-host"
))]
pub use load::load_json_cache_strict;
#[cfg(any(feature = "dashboard-host", feature = "sns-host"))]
pub use model::CachedJsonReport;
pub use model::{JsonCacheReport, LoadJsonCacheRequest};
